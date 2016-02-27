
TARGET=ssldaemon

.PHONY: default
default:
	GOPATH=$(PWD)/../.. go build -o $(TARGET) daemon/main.go

.PHONY: clean
clean:
	rm -f $(TARGET)
