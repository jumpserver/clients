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
	cmd := exec.Command("open", filePath)
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

func handleDB(protocol, command string) *exec.Cmd {
	if protocol == "postgresql" {
		command = structurePostgreSQLCommand(command)
	}
	cmd := awakenCommand(command)
	return cmd
}
