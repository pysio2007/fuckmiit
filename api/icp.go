package api

import (
	"context"
	"encoding/json"
)

type ICPQueryRequest struct {
	PageNum     string `json:"pageNum"`
	PageSize    string `json:"pageSize"`
	UnitName    string `json:"unitName"`
	ServiceType int    `json:"serviceType"`
}

type ICPQueryListResponse struct {
	Code    int                `json:"code"`
	Msg     string             `json:"msg"`
	Params  ICPQueryListParams `json:"params"`
	Success bool               `json:"success"`
}

type ICPQueryListParams struct {
	List []ICPQueryListItem `json:"list"`
}

type ICPQueryListItem struct {
	MainID    int64 `json:"mainId"`
	DomainID  int64 `json:"domainId"`
	ServiceID int64 `json:"serviceId"`
}

type ICPDetailRequest struct {
	MainID    int64 `json:"mainId"`
	DomainID  int64 `json:"domainId"`
	ServiceID int64 `json:"serviceId"`
}

type ICPDetailResponse struct {
	Code    int           `json:"code"`
	Msg     string        `json:"msg"`
	Params  ICPDetailItem `json:"params"`
	Success bool          `json:"success"`
}

type ICPDetailItem struct {
	ContentTypeName  string `json:"contentTypeName"`
	Domain           string `json:"domain"`
	DomainID         int64  `json:"domainId"`
	LeaderName       string `json:"leaderName"`
	LimitAccess      string `json:"limitAccess"`
	MainID           int64  `json:"mainId"`
	MainLicence      string `json:"mainLicence"`
	NatureName       string `json:"natureName"`
	ServiceID        int64  `json:"serviceId"`
	ServiceLicence   string `json:"serviceLicence"`
	UnitName         string `json:"unitName"`
	UpdateRecordTime string `json:"updateRecordTime"`
}

func (c *Client) QueryICPAbbreviateInfo(request ICPQueryRequest) ([]byte, error) {
	ctx, cancel := context.WithTimeout(context.Background(), c.httpClient.Timeout)
	defer cancel()

	return c.PostJSONRaw(ctx, "icpAbbreviateInfo/queryByCondition", request)
}

func (c *Client) QueryICPDetailByServiceIDAndDomainID(request ICPDetailRequest) ([]byte, error) {
	ctx, cancel := context.WithTimeout(context.Background(), c.httpClient.Timeout)
	defer cancel()

	return c.PostJSONRaw(ctx, "icpAbbreviateInfo/queryDetailByServiceIdAndDomainId", request)
}

func ParseICPQueryList(body []byte) (ICPQueryListResponse, error) {
	var response ICPQueryListResponse
	if err := json.Unmarshal(body, &response); err != nil {
		return ICPQueryListResponse{}, err
	}

	return response, nil
}

func ParseICPDetail(body []byte) (ICPDetailResponse, error) {
	var response ICPDetailResponse
	if err := json.Unmarshal(body, &response); err != nil {
		return ICPDetailResponse{}, err
	}

	return response, nil
}
