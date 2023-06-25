package awaken

import (
	"fmt"
	"go-client/global"
	"go-client/pkg/config"
	"io/ioutil"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

/*{
	"id": "f",
	"value": "q",
	"protocol": "ssh",
	"command": "xxx"
	"file": {
		"name": "name",
		"content": "content",
	}
}*/

type File struct {
	Name    string `json:"name"`
	Content string `json:"content"`
}

type Info struct {
	ID       string `json:"id"`
	Value    string `json:"value"`
	Protocol string `json:"protocol"`
	Command  string `json:"command"`
	File     `json:"file"`
}

type DBCommand struct {
	User     string `json:"user"`
	Password string `json:"password"`
	Host     string `json:"host"`
	Port     string `json:"port"`
	DBName   string `json:"dbname"`
}

type Rouse struct {
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
	filePath := filepath.Join(filepath.Dir(os.Args[0]), r.Name+".rdp")
	err := ioutil.WriteFile(filePath, []byte(r.Content), os.ModePerm)
	if err != nil {
		global.LOG.Error(err.Error())
		return
	}
	cmd := handleRDP(filePath)
	cmd.Run()
}

func (r *Rouse) HandleSSH() {
	currentPath := filepath.Dir(os.Args[0])
	cmd := handleSSH(r.Command, r.Value, currentPath)
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
	protocol := r.Protocol

	appConfig := config.GetConf()
	appsClients := appConfig.Apps
	for _, v := range appsClients {
		for _, pro := range v.Protocol {
			if protocol == pro && v.IsActive() {
				fmt.Println("error format: ", v)
			}
		}
	}

	switch protocol {
	case "rdp":
		r.HandleRDP()
	case "ssh":
		r.HandleSSH()
	case "mysql", "mariadb", "postgresql", "redis", "oracle", "sqlserver":
		r.HandleDB()
	}
}
