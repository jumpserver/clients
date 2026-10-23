package awaken

import (
	"strings"
	"testing"
)

func TestNavicatSharedMySQLWireProfile(t *testing.T) {
	for _, protocol := range []string{"mariadb", "maria", "mysql", "oracle", "sqlserver", "postgresql"} {
		t.Run(protocol, func(t *testing.T) {
			info := map[string]string{"protocol": protocol, "host": "proxy.example", "port": "5528", "username": "asset-token", "name": "root@db asset", "dbname": "app"}
			url := getNavicatURL(info)
			want := map[string]string{"mariadb": "mysql", "maria": "mysql", "mysql": "mysql", "oracle": "ora", "sqlserver": "mssql", "postgresql": "pgsql"}[protocol]
			if !strings.HasPrefix(url, "navicat://conn."+want+"?") {
				t.Fatalf("URL = %q, want Navicat profile %q", url, want)
			}
			for _, part := range []string{"Conn.Host=proxy.example", "Conn.Port=5528", "Conn.Username=asset-token"} {
				if !strings.Contains(url, part) {
					t.Fatalf("URL %q lost %q", url, part)
				}
			}
			if info["protocol"] != protocol {
				t.Fatal("Navicat URL generation changed the asset protocol")
			}
		})
	}
}
