package awaken

import (
	"fmt"
	"os/exec"
)

func awakenRDPCommand(filePath string) *exec.Cmd {
	cmd := exec.Command("open", filePath)
	return cmd
}

func awakenCommand(command string) *exec.Cmd {
	cmd := exec.Command(
		"osascript", "-s", "h", "-e",
		fmt.Sprintf(`tell application "Terminal" to do script "%s"`, command),
	)
	return cmd
}
