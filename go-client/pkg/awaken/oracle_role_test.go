package awaken

import (
	"encoding/json"
	"go-client/pkg/config"
	"strings"
	"testing"
)

func TestOracleLaunchRole(t *testing.T) {
	for _, enabled := range []bool{false, true} {
		r := &Rouse{}
		payload := `{"protocol":"oracle","connect_options":{"use_sysdba":false}}`
		if enabled {
			payload = strings.Replace(payload, "false", "true", 1)
		}
		if err := json.Unmarshal([]byte(payload), &r.Info); err != nil {
			t.Fatal(err)
		}
		app := &config.AppItem{Name: "dbeaver", ArgFormat: "-con name=test|driver=oracle|user=token"}
		args := getDatabaseArgFormat(r, app)
		if strings.Contains(args, "|auth=oracle_native|authProp.oracle.logon-as=SYSDBA") != enabled {
			t.Fatalf("DBeaver role mismatch: %s", args)
		}
		if strings.HasSuffix(getOracleCLIArgFormat(r), " as sysdba") != enabled {
			t.Fatal("SQLPlus role mismatch")
		}
		r.Protocol = "mysql"
		if getDatabaseArgFormat(r, app) != app.ArgFormat {
			t.Fatal("Oracle role leaked into another protocol")
		}
	}
}
