package awaken

import (
	"fmt"
	"os/exec"
	"path/filepath"
	"syscall"
)

func handleRDP(filePath string) *exec.Cmd {
	cmd := exec.Command("mstsc.exe", filePath)
	return cmd
}

func handleSSH(t *Token, currentPath string) *exec.Cmd {
	puttyPath := "putty.exe"
	if _, err := exec.LookPath("putty.exe"); err != nil {
		puttyPath = filepath.Join(currentPath, "putty.exe")
	}
	cmd := exec.Command(
		puttyPath, "-ssh",
		fmt.Sprintf("%s@%s", t.UserName, t.Ip), "-P", t.Port, "-pw", t.Password,
	)
	return cmd
}

func handleDB(protocol, command string) *exec.Cmd {
	cmd := exec.Command("cmd")
	cmd.SysProcAttr = &syscall.SysProcAttr{HideWindow: true, CmdLine: `/c start cmd /k ` + command}
	return cmd
}
