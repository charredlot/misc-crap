
GO_LDFLAGS='-extldflags "-static"'
GOPATH=$(shell pwd)/../../..

.PHONY: default
default:
	GOPATH=$(GOPATH) go get
	GOPATH=$(GOPATH) go build -ldflags $(GO_LDFLAGS) -o l2tunnel
