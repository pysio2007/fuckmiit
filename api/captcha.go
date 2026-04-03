package api

import (
	"context"
	"crypto/rand"
	"fmt"
	"net/http"
)

type GetCheckImagePointRequest struct {
	ClientUID string `json:"clientUid"`
}

type GetCheckImagePointResponse struct {
	Code    int                 `json:"code"`
	Msg     string              `json:"msg"`
	Params  GetCheckImageParams `json:"params"`
	Success bool                `json:"success"`
}

type GetCheckImageParams struct {
	SmallImage string `json:"smallImage"`
	BigImage   string `json:"bigImage"`
	UUID       string `json:"uuid"`
	Height     int    `json:"height"`
}

type CheckImageRequest struct {
	Key   string `json:"key"`
	Value string `json:"value"`
}

type CheckImageResponse struct {
	Code    int    `json:"code"`
	Msg     string `json:"msg"`
	Params  string `json:"params"`
	Success bool   `json:"success"`
}

func (c *Client) GetCheckImagePoint(clientUID string) (GetCheckImagePointResponse, error) {
	ctx, cancel := context.WithTimeout(context.Background(), c.httpClient.Timeout)
	defer cancel()

	payload := GetCheckImagePointRequest{ClientUID: clientUID}
	var response GetCheckImagePointResponse
	if err := c.PostJSON(ctx, "image/getCheckImagePoint", payload, &response); err != nil {
		return GetCheckImagePointResponse{}, err
	}
	if response.Code != http.StatusOK || !response.Success {
		return GetCheckImagePointResponse{}, fmt.Errorf("getCheckImagePoint rejected: code=%d msg=%s", response.Code, response.Msg)
	}

	return response, nil
}

func (c *Client) CheckImage(key string, value int) (CheckImageResponse, error) {
	response, err := c.TryCheckImage(key, value)
	if err != nil {
		return CheckImageResponse{}, err
	}
	if response.Code != http.StatusOK || !response.Success {
		return CheckImageResponse{}, fmt.Errorf("checkImage rejected: code=%d msg=%s", response.Code, response.Msg)
	}

	return response, nil
}

func (c *Client) TryCheckImage(key string, value int) (CheckImageResponse, error) {
	ctx, cancel := context.WithTimeout(context.Background(), c.httpClient.Timeout)
	defer cancel()

	payload := CheckImageRequest{
		Key:   key,
		Value: fmt.Sprintf("%d", value),
	}
	var response CheckImageResponse
	if err := c.PostJSON(ctx, "image/checkImage", payload, &response); err != nil {
		return CheckImageResponse{}, err
	}

	return response, nil
}

func NewClientUID() (string, error) {
	var raw [16]byte
	if _, err := rand.Read(raw[:]); err != nil {
		return "", err
	}

	raw[6] = (raw[6] & 0x0f) | 0x40
	raw[8] = (raw[8] & 0x3f) | 0x80

	return fmt.Sprintf(
		"point-%02x%02x%02x%02x-%02x%02x-%02x%02x-%02x%02x-%02x%02x%02x%02x%02x%02x",
		raw[0], raw[1], raw[2], raw[3],
		raw[4], raw[5],
		raw[6], raw[7],
		raw[8], raw[9],
		raw[10], raw[11], raw[12], raw[13], raw[14], raw[15],
	), nil
}

func (c *Client) SetSign(sign string) {
	c.SetHeader("Sign", sign)
}

func (c *Client) SetUUID(uuid string) {
	c.SetHeader("Uuid", uuid)
}
