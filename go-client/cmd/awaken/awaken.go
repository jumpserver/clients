package main

import (
	"encoding/base64"
	"encoding/json"
	"go-client/pkg/awaken"
	"net/url"
	"os"
	"runtime"
)

func main() {
	p := &awaken.Info{}
	if len(os.Args) != 1 {
		u, _ := url.Parse(os.Args[1])
		base64String := u.Hostname()
		decoded, _ := base64.StdEncoding.DecodeString(base64String)
		infoJson := string(decoded)
		json.Unmarshal([]byte(infoJson), p)
		r := awaken.Rouse{
			SysType: runtime.GOOS,
			Info:    *p,
		}
		r.Run()
	}
}

//go build -o JumpServerClient awaken.go
//CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build -ldflags="-H windowsgui" -o JumpServerClient.exe awaken.go
//CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o JumpServerClient awaken.go