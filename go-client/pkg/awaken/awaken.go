package awaken

import (
	"go-client/global"
	"go-client/pkg/config"
	"io/ioutil"
	"os"
	"path/filepath"
	"regexp"
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
	Name     string `json:"name"`
	Value    string `json:"value"`
	Protocol string `json:"protocol"`
	Host     string `json:"host"`
	Port     int    `json:"port"`
	Username string `json:"username"`
	Command  string `json:"command"`
	DBName   string `json:"dbname"`
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

func (r *Rouse) HandleRDP(appConfig *config.AppConfig) {
	removeCurRdpFile()
	filePath := filepath.Join(filepath.Dir(os.Args[0]), r.Name+".rdp")
	err := ioutil.WriteFile(filePath, []byte(r.Content), os.ModePerm)
	if err != nil {
		global.LOG.Error(err.Error())
		return
	}
	cmd := handleRDP(r, filePath, appConfig)
	cmd.Run()
}

func (r *Rouse) HandleSSH(appConfig *config.AppConfig) {
	currentPath := filepath.Dir(os.Args[0])
	cmd := handleSSH(r, currentPath, appConfig)
	cmd.Run()
}

func (r *Rouse) HandleDB(appConfig *config.AppConfig) {
	command := r.Command
	switch r.Protocol {
	case "mysql", "mariadb":
		command = structureMySQLCommand(command)
	case "postgresql":
		command = structurePostgreSQLCommand(command)
	case "redis":
		command = structureRedisCommand(command)
	}
	cmd := handleDB(r, command, appConfig)
	cmd.Run()
}

func (r *Rouse) Run() {
	protocol := r.Protocol
	appConfig := config.GetConf()

	switch protocol {
	case "rdp":
		r.HandleRDP(&appConfig)
	case "ssh":
		r.HandleSSH(&appConfig)
	case "mysql", "mariadb", "postgresql", "redis", "oracle", "sqlserver":
		r.HandleDB(&appConfig)
	}
}
