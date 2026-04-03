package api

import (
	"context"
	"crypto/md5"
	"encoding/hex"
	"fmt"
	"net/http"
	"net/url"
	"time"
)

const authSecret = "testtest"

type AuthResponse struct {
	Code    int        `json:"code"`
	Msg     string     `json:"msg"`
	Params  AuthParams `json:"params"`
	Success bool       `json:"success"`
}

type AuthParams struct {
	Expire   int64  `json:"expire"`
	Refresh  string `json:"refresh"`
	Business string `json:"bussiness"`
}

func (c *Client) Auth(timestamp int64) (AuthResponse, error) {
	ctx, cancel := context.WithTimeout(context.Background(), c.httpClient.Timeout)
	defer cancel()

	form := url.Values{}
	form.Set("authKey", buildAuthKey(authSecret, timestamp))
	form.Set("timeStamp", fmt.Sprintf("%d", timestamp))

	var response AuthResponse
	if err := c.PostFormJSON(ctx, "auth", form, &response); err != nil {
		return AuthResponse{}, err
	}
	if response.Code != http.StatusOK || !response.Success {
		return AuthResponse{}, fmt.Errorf("auth request rejected: code=%d msg=%s", response.Code, response.Msg)
	}
	c.SetToken(response.Params.Business)

	return response, nil
}

func buildAuthKey(secret string, timestamp int64) string {
	sum := md5.Sum([]byte(fmt.Sprintf("%s%d", secret, timestamp)))
	return hex.EncodeToString(sum[:])
}

func BuildAuthKey(timestamp int64) string {
	return buildAuthKey(authSecret, timestamp)
}

func DefaultTimeout() time.Duration {
	return defaultHTTPTimeout
}
