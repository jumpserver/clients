//go:build aix || darwin || dragonfly || freebsd || (js && wasm) || linux || netbsd || openbsd || solaris
// +build aix darwin dragonfly freebsd js,wasm linux netbsd openbsd solaris

package awaken

import (
	"fmt"
	"go-client/pkg/config"
	"os/exec"
	"path/filepath"
	"strings"
)

func handleRDP(r *Rouse, filePath string, cfg *config.AppConfig) *exec.Cmd {
	cmd := awakenRDPCommand(filePath)
	return cmd
}

func handleSSH(r *Rouse, currentPath string, cfg *config.AppConfig) *exec.Cmd {
	clientPath := filepath.Join(currentPath, "client")
	command := fmt.Sprintf("%s %s -P %s", clientPath, r.Command, r.Value)
	cmd := awakenCommand(command)
	return cmd
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

func structurePostgreSQLCommand(command string) string {
	command = strings.Trim(strings.ReplaceAll(command, "psql ", ""), `"`)
	db := &DBCommand{}
	for _, v := range strings.Split(command, " ") {
		tp, val := strings.Split(v, "=")[0], strings.Split(v, "=")[1]
		switch tp {
		case "user":
			db.User = val
		case "password":
			db.Password = val
		case "host":
			db.Host = val
		case "port":
			db.Port = val
		case "dbname":
			db.DBName = val
		}
	}
	command = fmt.Sprintf(
		"PGPASSWORD=%s psql -U %s -h %s -p %s -d %s",
		db.Password, db.User, db.Host, db.Port, db.DBName,
	)
	return command
}

func handleDB(r *Rouse, command string, cfg *config.AppConfig) *exec.Cmd {
	cmd := awakenCommand(command)
	return cmd
}
