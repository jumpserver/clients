package awaken

import (
	"fmt"
	"regexp"
	"strings"
)

// Magnus routes MariaDB through its server-first MySQL wire listener. Navicat's
// MariaDB-only profile rejects that greeting before it can use the asset token.
// The MySQL profile supports the shared wire protocol; the asset identity and
// audit protocol remain MariaDB.
func navicatWireProtocol(protocol string) string {
	if strings.EqualFold(protocol, "mariadb") || strings.EqualFold(protocol, "maria") {
		return "mysql"
	}
	return protocol
}

func getNavicatURL(connectInfo map[string]string) string {
	re := regexp.MustCompile(`@(.+)$`)
	matches := re.FindStringSubmatch(connectInfo["name"])
	name := connectInfo["username"]
	if len(matches) > 1 {
		name = matches[1]
	}
	url := fmt.Sprintf("navicat://conn.%s?Conn.Host=%s&Conn.Name=%s&Conn.Port=%s&Conn.Username=%s",
		navicatWireProtocol(connectInfo["protocol"]), connectInfo["host"], name, connectInfo["port"], connectInfo["username"])
	switch connectInfo["protocol"] {
	case "oracle":
		url = strings.Replace(url, "conn.oracle", "conn.ora", 1)
		url += fmt.Sprintf("&Conn.ServiceName=%s&Conn.ServiceNameType=ServiceName&Conn.ConnectionMode=Basic", connectInfo["dbname"])
	case "sqlserver":
		url = strings.Replace(url, "conn.sqlserver", "conn.mssql", 1)
		url += fmt.Sprintf("&Conn.AuthenticationType=Default&Conn.InitialDatabase=%s", connectInfo["dbname"])
	case "postgresql":
		url = strings.Replace(url, "conn.postgresql", "conn.pgsql", 1)
		url += fmt.Sprintf("&Conn.InitialDatabase=%s", connectInfo["dbname"])
	}

	pattern := regexp.MustCompile(`[\^(){}~]`)
	url = pattern.ReplaceAllStringFunc(url, func(match string) string {
		return fmt.Sprintf("{%s}", match)
	})
	return url
}
