package awaken

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"regexp"
	"strings"
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

type DBCommand struct {
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

func structureMySQLCommand(command string) string {
	command = strings.ReplaceAll(command, "mysql ", "")
	db := &DBCommand{}
	paramSlice := strings.Split(command, " ")
	for i, v := range paramSlice {
		if strings.HasPrefix(v, "-p") {
			db.Password = paramSlice[i][2:]
			continue
		}
		switch v {
		case "-u":
			db.User = paramSlice[i+1]
		case "-h":
			db.Host = paramSlice[i+1]
		case "-P":
			db.Port = paramSlice[i+1]
		}
	}
	db.DBName = paramSlice[len(paramSlice)-1]
	command = fmt.Sprintf(
		"mysql -u %s -p%s -h %s -P %s %s",
		db.User, db.Password, db.Host, db.Port, db.DBName,
	)
	return command
}

func structureRedisCommand(command string) string {
	command = strings.ReplaceAll(command, "redis-cli ", "")
	db := &DBCommand{}
	paramSlice := strings.Split(command, " ")
	for i, v := range paramSlice {
		switch v {
		case "-a":
			db.Password = paramSlice[i+1]
		case "-h":
			db.Host = paramSlice[i+1]
		case "-p":
			db.Port = paramSlice[i+1]
		}
	}
	cmd := fmt.Sprintf(
		"redis-cli -h %s -p %s -a %s",
		db.Host, db.Port, db.Password,
	)
	return cmd
}

func (r *Rouse) HandleDB() {
	command := r.Command
	switch r.Protocol {
	case "mysql", "mariadb":
		command = structureMySQLCommand(command)
	case "postgresql":
		command = structurePostgreSQLCommand(command)
	case "redis":
		command = structureRedisCommand(command)
	}
	cmd := handleDB(command)
	cmd.Run()
}
func (r *Rouse) Run() {
	protocol := r.Info.Protocol
	switch protocol {
	case "rdp":
		r.HandleRDP()
	case "ssh":
		r.HandleSSH()
	case "mysql", "mariadb", "postgresql", "redis", "oracle", "sqlserver":
		r.HandleDB()
	}
}
