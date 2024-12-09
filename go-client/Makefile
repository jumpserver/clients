NAME=go-client
BUILDDIR=build

BASEPATH := $(shell pwd)
CLIENTSRCFILE := $(BASEPATH)/cmd/awaken/
SSHCSRCFILE := $(BASEPATH)/cmd/sshc/

LDFLAGS=-w -s

CLIENTBUILD=CGO_ENABLED=0 go build -trimpath

build-client:
	GOARCH=amd64 GOOS=darwin $(CLIENTBUILD) -ldflags "$(LDFLAGS)" -o $(BUILDDIR)/darwin/JumpServerClient $(CLIENTSRCFILE)
	GOARCH=amd64 GOOS=darwin $(CLIENTBUILD) -ldflags "$(LDFLAGS)" -o $(BUILDDIR)/darwin/client $(SSHCSRCFILE)

	GOARCH=amd64 GOOS=windows $(CLIENTBUILD) -ldflags "$(LDFLAGS) -H windowsgui" -o $(BUILDDIR)/windows/JumpServerClient.exe $(CLIENTSRCFILE)
	GOARCH=386 GOOS=windows $(CLIENTBUILD) -ldflags "$(LDFLAGS) -H windowsgui" -o $(BUILDDIR)/windows/JumpServerClient32.exe $(CLIENTSRCFILE)

	GOARCH=amd64 GOOS=linux $(CLIENTBUILD) -ldflags "$(LDFLAGS)" -o $(BUILDDIR)/linux-amd64/JumpServerClient $(CLIENTSRCFILE)
	GOARCH=amd64 GOOS=linux $(CLIENTBUILD) -ldflags "$(LDFLAGS)" -o $(BUILDDIR)/linux-amd64/client $(SSHCSRCFILE)

	GOARCH=arm64 GOOS=linux $(CLIENTBUILD) -ldflags "$(LDFLAGS)" -o $(BUILDDIR)/linux-arm64/JumpServerClient $(CLIENTSRCFILE)
	GOARCH=arm64 GOOS=linux $(CLIENTBUILD) -ldflags "$(LDFLAGS)" -o $(BUILDDIR)/linux-arm64/client $(SSHCSRCFILE)

	mkdir -p $(BASEPATH)/../ui/bin/

	cp -R $(BASEPATH)/Scripts $(BUILDDIR)/darwin
	cp -R $(BUILDDIR)/* $(BASEPATH)/../ui/bin/
	cp $(BASEPATH)/config.json $(BASEPATH)/../ui/bin/
	cp $(BASEPATH)/putty.exe $(BASEPATH)/../ui/bin/windows/
	cp $(BASEPATH)/pkg/autoit/*.dll $(BASEPATH)/../ui/bin/windows/
