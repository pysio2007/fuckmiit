package fuckmiit

import (
	"fmt"
	"os"
	"time"

	"github.com/Mxmilu666/fuckmiit/api"
)

const (
	defaultServiceType = 1
	defaultPageNum     = ""
	defaultPageSize    = ""
	defaultTimeout     = 15 * time.Second
)

type DomainDetail = api.ICPDetailItem

// QueryDomain runs the full MIIT query flow for a domain and returns the
// final queryDetailByServiceIdAndDomainId JSON response body.
//
// Usage:
//
//	result, err := fuckmiit.QueryDomain("example.com")
//	result, err := fuckmiit.QueryDomain("example.com", true)
func QueryDomain(domain string, debug ...bool) ([]byte, error) {
	if domain == "" {
		return nil, fmt.Errorf("domain is required")
	}

	debugEnabled := len(debug) > 0 && debug[0]
	timestamp := time.Now().Unix()
	client := api.NewClient(defaultTimeout)

	debugf(debugEnabled, "step=auth timestamp=%d", timestamp)
	authResponse, err := client.Auth(timestamp)
	if err != nil {
		return nil, err
	}
	debugf(debugEnabled, "step=auth success=true expire=%d", authResponse.Params.Expire)

	clientUID, err := api.NewClientUID()
	if err != nil {
		return nil, err
	}
	debugf(debugEnabled, "step=getCheckImagePoint clientUid=%s", clientUID)

	challenge, err := client.GetCheckImagePoint(clientUID)
	if err != nil {
		return nil, err
	}
	debugf(debugEnabled, "step=getCheckImagePoint success=true captchaUUID=%s height=%d", challenge.Params.UUID, challenge.Params.Height)

	_, checkResponse, err := solveCaptcha(client, challenge.Params.UUID, challenge.Params.BigImage, challenge.Params.Height, debugEnabled)
	if err != nil {
		return nil, err
	}

	client.SetSign(checkResponse.Params)
	client.SetUUID(challenge.Params.UUID)
	debugf(debugEnabled, "step=query endpoint=icpAbbreviateInfo/queryByCondition unitName=%s serviceType=%d", domain, defaultServiceType)

	queryResponse, err := client.QueryICPAbbreviateInfo(api.ICPQueryRequest{
		PageNum:     defaultPageNum,
		PageSize:    defaultPageSize,
		UnitName:    domain,
		ServiceType: defaultServiceType,
	})
	if err != nil {
		return nil, err
	}

	queryList, err := api.ParseICPQueryList(queryResponse)
	if err != nil {
		return nil, err
	}
	if len(queryList.Params.List) == 0 {
		return nil, fmt.Errorf("queryByCondition returned no records for %s", domain)
	}

	item := queryList.Params.List[0]
	debugf(debugEnabled, "step=queryDetail endpoint=icpAbbreviateInfo/queryDetailByServiceIdAndDomainId mainId=%d domainId=%d serviceId=%d", item.MainID, item.DomainID, item.ServiceID)

	detailResponse, err := client.QueryICPDetailByServiceIDAndDomainID(api.ICPDetailRequest{
		MainID:    item.MainID,
		DomainID:  item.DomainID,
		ServiceID: item.ServiceID,
	})
	if err != nil {
		return nil, err
	}

	return detailResponse, nil
}

// QueryDomainDetail runs the full MIIT query flow and returns the final detail
// response as a single DomainDetail struct.
func QueryDomainDetail(domain string, debug ...bool) (DomainDetail, error) {
	body, err := QueryDomain(domain, debug...)
	if err != nil {
		return DomainDetail{}, err
	}

	return parseDomainDetail(body)
}

func parseDomainDetail(body []byte) (DomainDetail, error) {
	response, err := api.ParseICPDetail(body)
	if err != nil {
		return DomainDetail{}, err
	}
	if !response.Success || response.Code != 200 {
		return DomainDetail{}, fmt.Errorf("detail response rejected: code=%d msg=%s", response.Code, response.Msg)
	}

	return response.Params, nil
}

func debugf(enabled bool, format string, args ...any) {
	if !enabled {
		return
	}
	fmt.Fprintf(os.Stderr, "debug: "+format+"\n", args...)
}

func solveCaptcha(client *api.Client, captchaUUID, bigImage string, topHint int, debug bool) (rect, api.CheckImageResponse, error) {
	box, err := detectSquareBase64WithHint(bigImage, topHint)
	if err != nil {
		return rect{}, api.CheckImageResponse{}, err
	}
	debugf(debug, "step=detect left=%d top=%d right=%d bottom=%d", box.left, box.top, box.right, box.bottom)

	for _, left := range candidateOffsets(box.left, 12) {
		debugf(debug, "step=checkImage attempt_left=%d", left)
		response, err := client.TryCheckImage(captchaUUID, left)
		if err != nil {
			return rect{}, api.CheckImageResponse{}, err
		}
		if response.Code == 200 && response.Success {
			box.right += left - box.left
			box.left = left
			debugf(debug, "step=checkImage success=true sign_len=%d", len(response.Params))
			return box, response, nil
		}
	}

	return box, api.CheckImageResponse{}, fmt.Errorf("checkImage failed around detected left=%d", box.left)
}

func candidateOffsets(center, radius int) []int {
	seen := make(map[int]struct{}, radius*2+1)
	offsets := make([]int, 0, radius*2+1)

	add := func(v int) {
		if v < 0 {
			return
		}
		if _, ok := seen[v]; ok {
			return
		}
		seen[v] = struct{}{}
		offsets = append(offsets, v)
	}

	add(center)
	for delta := 1; delta <= radius; delta++ {
		add(center - delta)
		add(center + delta)
	}

	return offsets
}
