package manager

import (
    "net"
)

type ClientMgr struct {
    clients map[string]net.Conn
}

func NewClientMgr() *ClientMgr{
    return &ClientMgr{
        clients: make(map[string]net.Conn),
    }
}

func (mgr *ClientMgr) HandleNewClientConn(conn net.Conn) {
}
