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
	"strings"
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
func StructureMacCommand(command string) string {
	command = strings.Trim(strings.ReplaceAll(command, "psql ", ""), `"`)
	psql := &PostgresqlInfo{}
	for _, v := range strings.Split(command, " ") {
		tp, val := strings.Split(v, "=")[0], strings.Split(v, "=")[1]
		switch tp {
		case "user":
			psql.User = val
		case "password":
			psql.Password = val
		case "host":
			psql.Host = val
		case "port":
			psql.Port = val
		case "dbname":
			psql.DBName = val
		}
	}
	macCommand := fmt.Sprintf(
		"PGPASSWORD=%s psql -U %s -h %s -p %s -d %s",
		psql.Password, psql.User, psql.Host, psql.Port, psql.DBName,
	)
	return macCommand
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
	Command  string `json:"command"`
	Config   string `json:"config"`
}

type PostgresqlInfo struct {
	User     string `json:"user"`
	Password string `json:"password"`
	Host     string `json:"host"`
	Port     string `json:"port"`
	DBName   string `json:"dbname"`
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

	if err = cmd.Run(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

func (r *Rouse) HandleSSH() {
	t := &Token{}
	json.Unmarshal([]byte(r.Token), t)

	if r.SysType == "windows" {
		puttyPath := "putty.exe"
		if _, err := exec.LookPath("putty.exe"); err != nil {
			puttyPath = filepath.Join(filepath.Dir(os.Args[0]), "putty.exe")
		}
		cmd = exec.Command(
			puttyPath, "-ssh",
			fmt.Sprintf("%s@%s", t.UserName, t.Ip), "-P", t.Port, "-pw", t.Password,
		)
	} else {
		filePath := filepath.Join(filepath.Dir(os.Args[0]), "client")
		sshCmd := fmt.Sprintf(
			"%s ssh %s@%s -p %s -P %s", filePath, t.UserName, t.Ip, t.Port, t.Password,
		)
		cmd = exec.Command(
			"osascript", "-s", "h", "-e",
			fmt.Sprintf(`tell application "Terminal" to do script "%s"`, sshCmd),
		)
	}
	cmd.Run()
}

func (r *Rouse) HandleDB() {
	if r.SysType == "windows" {
		cmd = exec.Command("cmd")
		//cmd.SysProcAttr = &syscall.SysProcAttr{HideWindow: true, CmdLine: `/c start cmd /k ` + r.Command}
	} else {
		command := r.Command
		if r.Protocol == "postgresql" {
			command = StructureMacCommand(command)
		}
		cmd = exec.Command(
			"osascript", "-s", "h", "-e",
			fmt.Sprintf(`tell application "Terminal" to do script "%s"`, command),
		)
	}
	cmd.Run()
}
func (r *Rouse) Run() {
	protocol := r.Info.Protocol
	switch protocol {
	case "rdp":
		r.HandleRDP()
	case "ssh":
		r.HandleSSH()
	default:
		// 找不到默认走DB
		r.HandleDB()
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

//CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build -ldflags="-H windowsgui" -o JumpServerClient.exe awaken.go
