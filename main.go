package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
)

var cmd *exec.Cmd

type Info struct {
	Protocol string `json:"protocol"`
	UserName string `json:"username"`
	Token    string `json:"token"`
	Config   string `json:"config"`
}

type Rouse struct {
	SysType string
	Info
}

func (r *Rouse) HandleRDP() {
	dir, _ := os.Getwd()
	err := ioutil.WriteFile(filepath.Join(dir, "xx.rdp"), []byte(r.Info.Config), 0777)
	if err != nil {
		fmt.Println("write file failed, err:", err)
		return
	}
	if r.SysType == "windows" {
		cmd = exec.Command("mstsc.exe", filepath.Join(dir, "xx.rdp"))
	} else {
		cmd = exec.Command("open", filepath.Join(dir, "xx.rdp"))
	}

	if _, err = cmd.Output(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

func (r *Rouse) HandleSSH() {
	fmt.Println("use ssh")
}

func (r *Rouse) Run() {
	protocol := r.Info.Protocol
	switch protocol {
	case "rdp":
		r.HandleRDP()
	case "ssh":
		r.HandleSSH()
	default:
		r.HandleRDP()
	}
}

func main() {
	p := &Info{}
	if len(os.Args) != 1 {
		base64String := strings.Replace(os.Args[1], "jms://", "", 1)
		decoded, _ := base64.StdEncoding.DecodeString(base64String)
		infoJson := string(decoded)
		json.Unmarshal([]byte(infoJson), p)
		r := Rouse{
			SysType: runtime.GOOS,
			Info:    *p,
		}
		r.Run()
	}

}
