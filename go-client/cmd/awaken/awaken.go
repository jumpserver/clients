package main

import (
	"encoding/base64"
	"encoding/json"
	"go-client/global"
	"go-client/pkg/awaken"
	"go-client/pkg/logger"
	"go.uber.org/zap"
	"net/url"
	"os"
)

func main() {
	p := &awaken.Info{}
	if len(os.Args) != 1 {
		global.LOG = logger.InitLogger()
		zap.ReplaceGlobals(global.LOG)
		u, _ := url.Parse(os.Args[1])
		base64String := u.Hostname()
		decoded, _ := base64.StdEncoding.DecodeString(base64String)
		infoJson := string(decoded)
		json.Unmarshal([]byte(infoJson), p)
		r := awaken.Rouse{
			Info:    *p,
		}
		r.Run()
	}
}

//go build -o JumpServerClient awaken.go
//CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build -ldflags="-H windowsgui" -o JumpServerClient.exe awaken.go
//CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o JumpServerClient awaken.go