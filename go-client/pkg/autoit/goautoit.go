//go:build windows
// +build windows

package autoit

import (
	"os"
	"path/filepath"
	"syscall"
	"unsafe"
)

const (
	SWShowNormal       = 1
	INTDEFAULT         = -2147483647
	DefaultMouseButton = "left"
)

// HWND -- window handle
type HWND uintptr

// RECT -- http://msdn.microsoft.com/en-us/library/windows/desktop/dd162897.aspx
type RECT struct {
	Left, Top, Right, Bottom int32
}

// POINT --
type POINT struct {
	X, Y int32
}

var (
	dll64          *syscall.LazyDLL
	controlClick   *syscall.LazyProc
	controlSend    *syscall.LazyProc
	controlSetText *syscall.LazyProc
	run            *syscall.LazyProc
	winWaitActive  *syscall.LazyProc
	winWait        *syscall.LazyProc
)

func LoadAuto() {
	autoItX3 := "AutoItX3_x64.dll"
	if (^uint(0) >> 63) == 0 {
		autoItX3 = "AutoItX3.dll"
	}
	dll64 := syscall.NewLazyDLL(filepath.Join(filepath.Dir(os.Args[0]), autoItX3))
	controlClick = dll64.NewProc("AU3_ControlClick")
	controlSend = dll64.NewProc("AU3_ControlSend")
	controlSetText = dll64.NewProc("AU3_ControlSetText")
	run = dll64.NewProc("AU3_Run")
	winWaitActive = dll64.NewProc("AU3_WinWaitActive")
	winWait = dll64.NewProc("AU3_WinWait")
}

// Run -- Run a windows program
// flag 3(max) 6(min) 9(normal) 0(hide)
func Run(szProgram string, args ...interface{}) int {
	var szDir string
	var flag int
	var ok bool
	if len(args) == 0 {
		szDir = ""
		flag = SWShowNormal
	} else if len(args) == 1 {
		if szDir, ok = args[0].(string); !ok {
			panic("szDir must be a string")
		}
		flag = SWShowNormal
	} else if len(args) == 2 {
		if szDir, ok = args[0].(string); !ok {
			panic("szDir must be a string")
		}
		if flag, ok = args[1].(int); !ok {
			panic("flag must be a int")
		}
	} else {
		panic("Too more parameter")
	}
	pid, _, lastErr := run.Call(strPtr(szProgram), strPtr(szDir), intPtr(flag))
	if int(pid) == 0 {
		println(lastErr)
	}
	return int(pid)
}

// WinWait -- wait window to active
func WinWait(szTitle string, args ...interface{}) int {
	var szText string
	var nTimeout int
	var ok bool
	if len(args) == 0 {
		szText = ""
		nTimeout = 0
	} else if len(args) == 1 {
		if szText, ok = args[0].(string); !ok {
			panic("szText must be a string")
		}
		nTimeout = 0
	} else if len(args) == 2 {
		if szText, ok = args[0].(string); !ok {
			panic("szText must be a string")
		}
		if nTimeout, ok = args[1].(int); !ok {
			panic("nTimeout must be a int")
		}
	} else {
		panic("Too more parameter")
	}

	handle, _, lastErr := winWait.Call(strPtr(szTitle), strPtr(szText), intPtr(nTimeout))
	if int(handle) == 0 {
		println(lastErr)
	}
	return int(handle)
}

// ControlClick -- Sends a mouse click command to a given control.
func ControlClick(title, text, control string, args ...interface{}) int {
	var button string
	var x, y, nClicks int
	var ok bool

	if len(args) == 0 {
		button = DefaultMouseButton
		nClicks = 1
		x = INTDEFAULT
		y = INTDEFAULT
	} else if len(args) == 1 {
		if button, ok = args[0].(string); !ok {
			panic("button must be a string")
		}
		nClicks = 1
		x = INTDEFAULT
		y = INTDEFAULT
	} else if len(args) == 2 {
		if button, ok = args[0].(string); !ok {
			panic("button must be a string")
		}
		if nClicks, ok = args[1].(int); !ok {
			panic("nClicks must be a int")
		}
		x = INTDEFAULT
		y = INTDEFAULT
	} else if len(args) == 4 {
		if button, ok = args[0].(string); !ok {
			panic("button must be a string")
		}
		if nClicks, ok = args[1].(int); !ok {
			panic("nClicks must be a int")
		}
		if x, ok = args[2].(int); !ok {
			panic("x must be a int")
		}
		if y, ok = args[3].(int); !ok {
			panic("y must be a int")
		}
	} else {
		panic("Error parameters")
	}
	ret, _, lastErr := controlClick.Call(strPtr(title), strPtr(text), strPtr(control), strPtr(button), intPtr(nClicks), intPtr(x), intPtr(y))
	if int(ret) == 0 {
		println(lastErr)
	}
	return int(ret)
}

// WinWaitActive ( "title" [, "text"]) int
func WinWaitActive(szTitle string, args ...interface{}) int {
	var szText string
	var nTimeout int
	var ok bool
	if len(args) == 0 {
		szText = ""
		nTimeout = 0
	} else if len(args) == 1 {
		if szText, ok = args[0].(string); !ok {
			panic("szText must be a string")
		}
		nTimeout = 0
	} else if len(args) == 2 {
		if szText, ok = args[0].(string); !ok {
			panic("szText must be a string")
		}
		if nTimeout, ok = args[1].(int); !ok {
			panic("nTimeout must be a int")
		}
	} else {
		panic("Too more parameter")
	}

	ret, _, lastErr := winWaitActive.Call(strPtr(szTitle), strPtr(szText), intPtr(nTimeout))
	if int(ret) == 0 {
		println(lastErr)
	}
	return int(ret)
}

// ControlSend -- Sends a string of characters to a control.
func ControlSend(title, text, control, sendText string, args ...interface{}) int {
	var nMode int
	var ok bool
	if len(args) == 0 {
		nMode = 0
	} else if len(args) == 1 {
		if nMode, ok = args[0].(int); !ok {
			panic("nMode must be a int")
		}
	} else {
		panic("Too more parameter")
	}
	ret, _, lastErr := controlSend.Call(strPtr(title), strPtr(text), strPtr(control), strPtr(sendText), intPtr(nMode))
	if int(ret) == 0 {
		println(lastErr)
	}
	return int(ret)
}

// ControlSetText -- Sets text of a control.
func ControlSetText(title, text, control, newText string) int {
	ret, _, lastErr := controlSetText.Call(strPtr(title), strPtr(text), strPtr(control), strPtr(newText))
	if int(ret) == 0 {
		println(lastErr)
	}
	return int(ret)
}

func findTermChr(buff []uint16) int {
	for i, char := range buff {
		if char == 0x0 {
			return i
		}
	}
	panic("not supposed to happen")
}

func intPtr(n int) uintptr {
	return uintptr(n)
}

func strPtr(s string) uintptr {
	return uintptr(unsafe.Pointer(syscall.StringToUTF16Ptr(s)))
}
