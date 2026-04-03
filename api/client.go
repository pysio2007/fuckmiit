package api

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/cookiejar"
	"net/url"
	"strings"
	"time"
)

const (
	baseURL            = "https://hlwicpfwc.miit.gov.cn/icpproject_query/api/"
	serviceHost        = "hlwicpfwc.miit.gov.cn"
	defaultOrigin      = "https://beian.miit.gov.cn"
	defaultReferer     = "https://beian.miit.gov.cn/"
	defaultUserAgent   = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36 Edg/146.0.0.0"
	defaultAccept      = "application/json, text/plain, */*"
	defaultLanguage    = "zh-CN,zh-HK;q=0.9,zh;q=0.8,en-US;q=0.7,en;q=0.6"
	defaultConnection  = "keep-alive"
	formContentType    = "application/x-www-form-urlencoded"
	jsonContentType    = "application/json"
	defaultHTTPTimeout = 15 * time.Second
)

type Client struct {
	baseURL    string
	httpClient *http.Client
	headers    http.Header
}

func NewClient(timeout time.Duration) *Client {
	if timeout <= 0 {
		timeout = defaultHTTPTimeout
	}

	jar, err := cookiejar.New(nil)
	if err != nil {
		panic(err)
	}

	headers := make(http.Header)
	headers.Set("Origin", defaultOrigin)
	headers.Set("Referer", defaultReferer)
	headers.Set("User-Agent", defaultUserAgent)
	headers.Set("Accept", defaultAccept)
	headers.Set("Accept-Language", defaultLanguage)
	headers.Set("Connection", defaultConnection)

	return &Client{
		baseURL: baseURL,
		httpClient: &http.Client{
			Timeout: timeout,
			Jar:     jar,
		},
		headers: headers,
	}
}

func (c *Client) SetHeader(key, value string) {
	if value == "" {
		c.headers.Del(key)
		return
	}
	c.headers.Set(key, value)
}

func (c *Client) SetToken(token string) {
	c.SetHeader("Token", token)
}

func (c *Client) SetAuthorization(authorization string) {
	c.SetHeader("Authorization", authorization)
}

func (c *Client) NewRequest(ctx context.Context, method, path string, body io.Reader) (*http.Request, error) {
	req, err := http.NewRequestWithContext(ctx, method, c.resolveURL(path), body)
	if err != nil {
		return nil, err
	}

	req.Host = serviceHost
	for key, values := range c.headers {
		for _, value := range values {
			req.Header.Add(key, value)
		}
	}

	return req, nil
}

func (c *Client) PostFormJSON(ctx context.Context, path string, form url.Values, out any) error {
	req, err := c.NewRequest(ctx, http.MethodPost, path, strings.NewReader(form.Encode()))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", formContentType)

	return c.doJSON(req, out)
}

func (c *Client) PostJSON(ctx context.Context, path string, payload any, out any) error {
	body, err := json.Marshal(payload)
	if err != nil {
		return err
	}

	req, err := c.NewRequest(ctx, http.MethodPost, path, bytes.NewReader(body))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", jsonContentType)

	return c.doJSON(req, out)
}

func (c *Client) PostJSONRaw(ctx context.Context, path string, payload any) ([]byte, error) {
	body, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}

	req, err := c.NewRequest(ctx, http.MethodPost, path, bytes.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", jsonContentType)

	return c.doRaw(req)
}

func (c *Client) doJSON(req *http.Request, out any) error {
	body, err := c.doRaw(req)
	if err != nil {
		return err
	}
	if err := json.Unmarshal(body, out); err != nil {
		return fmt.Errorf("decode response failed: %w body=%s", err, strings.TrimSpace(string(body)))
	}

	return nil
}

func (c *Client) doRaw(req *http.Request) ([]byte, error) {
	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("request failed: status=%d body=%s", resp.StatusCode, strings.TrimSpace(string(body)))
	}

	return body, nil
}

func (c *Client) resolveURL(path string) string {
	if strings.HasPrefix(path, "http://") || strings.HasPrefix(path, "https://") {
		return path
	}
	return c.baseURL + strings.TrimPrefix(path, "/")
}
