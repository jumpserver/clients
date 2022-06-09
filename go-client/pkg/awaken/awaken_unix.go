//go:build aix || darwin || dragonfly || freebsd || (js && wasm) || linux || netbsd || openbsd || solaris
// +build aix darwin dragonfly freebsd js,wasm linux netbsd openbsd solaris

package awaken

import (
	"fmt"
	"os/exec"
	"path/filepath"
	"strings"
)

func handleRDP(filePath string) *exec.Cmd {
	cmd := awakenRDPCommand(filePath)
	return cmd
}

func handleSSH(t *Token, currentPath string) *exec.Cmd {
	clientPath := filepath.Join(currentPath, "client")
	command := fmt.Sprintf(
		"%s ssh %s@%s -p %s -P %s", clientPath, t.UserName, t.Ip, t.Port, t.Password,
	)
	cmd := awakenCommand(command)
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

func handleDB(command string) *exec.Cmd {
	cmd := awakenCommand(command)
	return cmd
}
