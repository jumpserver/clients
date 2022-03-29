package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
)

var cmd *exec.Cmd

/*{
	"filename": "{}-{}-jumpserver".format(username, name),
	"protocol": "ssh",
	"username": "laoguang",
	"token": "xxx",
	"config": "full address:s:rdjumpserver.fit2cloud.com:33390"
}*/

func RemoveCurRdpFile() {
	re := regexp.MustCompile(".*\\.rdp$")
	dir := filepath.Dir(os.Args[0])
	rd, _ := ioutil.ReadDir(dir)
	for _, v := range rd {
		if !v.IsDir() && re.MatchString(v.Name()) {
			os.Remove(filepath.Join(dir, v.Name()))
		}
	}
}

type Token struct {
	Ip       string `json:"ip"`
	Port     string `json:"port"`
	UserName string `json:"username"`
	Password string `json:"password"`
}

type Info struct {
	FileName string `json:"filename"`
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
	RemoveCurRdpFile()
	filePath := filepath.Join(filepath.Dir(os.Args[0]), r.Info.FileName+".rdp")
	err := ioutil.WriteFile(filePath, []byte(r.Info.Config), os.ModePerm)
	if err != nil {
		fmt.Println("write file failed, err:", err)
		return
	}
	if r.SysType == "windows" {
		cmd = exec.Command("mstsc.exe", filePath)
	} else {
		cmd = exec.Command("open", filePath)
	}

	if _, err = cmd.Output(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

func (r *Rouse) HandleSSH() {
	t := &Token{}
	json.Unmarshal([]byte(r.Token), t)
	filePath := filepath.Join(filepath.Dir(os.Args[0]), "client")
	sshCmd := fmt.Sprintf(
		"%s ssh %s@%s -p %s -P %s", filePath, t.UserName, t.Ip, t.Port, t.Password,
	)
	c := exec.Command(
		"osascript",
		"-s", "h", "-e",
		fmt.Sprintf(`tell application "Terminal" to do script "%s"`, sshCmd),
	)
	if err := c.Run(); err != nil {
		fmt.Println("Error: ", err)
	}
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
		u, _ := url.Parse(os.Args[1])
		base64String := u.Hostname()
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
