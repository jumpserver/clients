package awaken

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"regexp"
)

/*{
	"filename": "{}-{}-jumpserver".format(username, name),
	"protocol": "ssh",
	"username": "laoguang",
	"token": "xxx",
	"config": "full address:s:rdjumpserver.fit2cloud.com:33390"
}*/

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

func removeCurRdpFile() {
	re := regexp.MustCompile(".*\\.rdp$")
	dir := filepath.Dir(os.Args[0])
	rd, _ := ioutil.ReadDir(dir)
	for _, v := range rd {
		if !v.IsDir() && re.MatchString(v.Name()) {
			os.Remove(filepath.Join(dir, v.Name()))
		}
	}
}

func (r *Rouse) HandleRDP() {
	removeCurRdpFile()
	filePath := filepath.Join(filepath.Dir(os.Args[0]), r.Info.FileName+".rdp")
	err := ioutil.WriteFile(filePath, []byte(r.Info.Config), os.ModePerm)
	if err != nil {
		fmt.Println("write file failed, err:", err)
		return
	}

	cmd := handleRDP(filePath)
	cmd.Run()
}

func (r *Rouse) HandleSSH() {
	t := &Token{}
	json.Unmarshal([]byte(r.Token), t)
	currentPath := filepath.Dir(os.Args[0])
	cmd := handleSSH(t, currentPath)
	cmd.Run()
}

func (r *Rouse) HandleDB() {
	cmd := handleDB(r.Protocol, r.Command)
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
