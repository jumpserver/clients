package awaken

import (
	"go-client/pkg/config"
	"strings"
	"testing"
)

func TestOracleMacLaunchArguments(t *testing.T) {
	for _, enabled := range []bool{false, true} {
		r := &Rouse{Info: Info{Protocol: "oracle", Name: "test", ConnectOptions: ConnectOptions{UseSysDBA: enabled}, Token: Token{ID: "proxy-token", Value: "synthetic-secret"}, Endpoint: Endpoint{Host: "localhost", Port: 5525}, Asset: Asset{DBInfo: DBInfo{DBName: "ORCL"}}}}
		for _, name := range []string{"dbeaver", "terminal"} {
			app := config.AppItem{Name: name, DisplayName: "Terminal", Path: "/usr/bin/true", MatchFirst: []string{"oracle"}, IsSet: true, IsInternal: name == "terminal", ArgFormat: "-con driver=oracle|user={username}|database={dbname}|host={host}|port={port}"}
			cfg := &config.AppConfig{MacOS: config.AppType{Databases: []config.AppItem{app}}}
			cmd, err := awakenDBCommand(r, cfg)
			if err != nil || cmd == nil {
				t.Fatalf("%s: %v", name, err)
			}
			args := strings.Join(cmd.Args, " ")
			if name == "dbeaver" {
				if !strings.Contains(args, "database=proxy-token") || strings.Contains(args, "oracle.logon-as=SYSDBA") != enabled {
					t.Fatalf("unexpected DBeaver arguments: %s", args)
				}
			} else if !strings.Contains(args, "localhost:5525/proxy-token") || strings.Contains(args, " as sysdba") != enabled {
				t.Fatal("unexpected terminal SQLPlus arguments")
			}
		}
	}
}
