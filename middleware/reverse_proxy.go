package middleware

import (
	"github.com/gin-gonic/gin"
	"net/http/httputil"
	"net/url"
)

func ReverseProxy(url *url.URL) gin.HandlerFunc {
	return func(c *gin.Context) {
		req := c.Request
		res := c.Writer

		req.Host = url.Host
		req.URL.Host = url.Host
		req.URL.Scheme = url.Scheme

		httputil.NewSingleHostReverseProxy(url).ServeHTTP(res, req)
	}
}
