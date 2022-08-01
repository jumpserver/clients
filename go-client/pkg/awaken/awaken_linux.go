package awaken

import (
	"fmt"
	"go-client/global"
	"os/exec"
)

func awakenRDPCommand(filePath string) *exec.Cmd {
	global.LOG.Debug(filePath)
	cmd := exec.Command("remmina", filePath)
	return cmd
}

func awakenCommand(command string) *exec.Cmd {
	cmd := exec.Command(
		"gnome-terminal", "--", "bash", "-c",
		fmt.Sprintf("%s; exec bash -i", command),
	)
	return cmd
}
