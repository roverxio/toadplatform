package server

import (
	"github.com/gin-gonic/gin"
	"net/url"
	"os"
	"roverx-rpc/middleware"
)

func InitServer() {
	router := gin.New()
	// remote server
	nodeUrl, _ := url.Parse(os.Getenv("NODE_URL"))
	// proxy middleware
	router.Use(middleware.ReverseProxy(nodeUrl))
	err := router.Run(os.Getenv("PORT"))
	if err != nil {
		return
	}
}
