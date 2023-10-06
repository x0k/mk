package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"regexp"
	"strings"
)

const (
	RootScope = "root"
)

var receiptNameRegExp, receiptNameRegExpError = regexp.Compile(`^[A-Za-z][0-9A-Za-z\t _-]*:$`)
var scopeIdentRegExp, scopeIdentRegExpError = regexp.Compile(`^([ \t]+)`)

type StringWriteCloser interface {
	Write(line string) (int, error)
	io.Closer
}

type StdWriteCloser struct{}

func (StdWriteCloser) Write(line string) (int, error) {
	return fmt.Println(line)
}

func (StdWriteCloser) Close() error {
	return nil
}

type CmdWriteCloser struct {
	cmd       *exec.Cmd
	stdInPipe io.WriteCloser
}

func (o *CmdWriteCloser) Write(line string) (int, error) {
	return o.stdInPipe.Write([]byte(line + "\n"))
}

func (o *CmdWriteCloser) Close() error {
	err := o.stdInPipe.Close()
	if err != nil {
		return err
	}
	return o.cmd.Wait()
}

type ScopedWriteCloser struct {
	currentScope            string
	targetScope             string
	shouldDefineScopeIndent bool
	currentScopeIndent      string
	out                     StringWriteCloser
}

type ScopedWriteCloserScopeChangedError struct {
	scopedWriteCloser *ScopedWriteCloser
}

func (e ScopedWriteCloserScopeChangedError) Error() string {
	return e.scopedWriteCloser.currentScope
}

type ScopedWriteCloserScopeMismatchError struct {
	scopedWriteCloser *ScopedWriteCloser
}

func (e ScopedWriteCloserScopeMismatchError) Error() string {
	return e.scopedWriteCloser.currentScope
}

func (o *ScopedWriteCloser) writeScopedLine(line string) (int, error) {
	if o.currentScope == o.targetScope {
		return o.out.Write(strings.TrimPrefix(line, o.currentScopeIndent))
	}
	return 0, &ScopedWriteCloserScopeMismatchError{
		scopedWriteCloser: o,
	}
}

func (o *ScopedWriteCloser) scopeChanged(line string) (int, error) {
	if receiptNameRegExp.MatchString(line) {
		o.currentScope = line[:len(line)-1]
		o.shouldDefineScopeIndent = true
	} else {
		o.currentScope = RootScope
	}
	return 0, &ScopedWriteCloserScopeChangedError{
		scopedWriteCloser: o,
	}
}

func (o *ScopedWriteCloser) Write(line string) (int, error) {
	if o.currentScope == RootScope {
		if receiptNameRegExp.MatchString(line) {
			o.currentScope = line[:len(line)-1]
			o.shouldDefineScopeIndent = true
			return 0, &ScopedWriteCloserScopeChangedError{
				scopedWriteCloser: o,
			}
		}
		return o.out.Write(line)
	}
	// o.currentScope != RootScope
	if o.shouldDefineScopeIndent {
		o.shouldDefineScopeIndent = false
		matches := scopeIdentRegExp.FindStringSubmatch(line)
		if len(matches) == 2 {
			o.currentScopeIndent = matches[1]
			return o.writeScopedLine(line)
		}
		return o.scopeChanged(line)
	}
	if strings.HasPrefix(line, o.currentScopeIndent) {
		return o.writeScopedLine(line)
	}
	return o.scopeChanged(line)
}

func makeOutput(line string) (StringWriteCloser, error) {
	if strings.HasPrefix(line, "#!") {
		parts := strings.Fields(line)
		cmd := exec.Command(parts[0], parts[1:]...)
		cmd.Stdin = os.Stdin
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		cmd.Start()
		stdInPipe, err := cmd.StdinPipe()
		if err != nil {
			return nil, err
		}
		return &CmdWriteCloser{
			cmd:       cmd,
			stdInPipe: stdInPipe,
		}, nil
	} else {
		return &StdWriteCloser{}, nil
	}
}

type AppState struct {
	targetReceipt          string
	currentReceiptIsTarget bool
}

func (s *AppState) isCookingDone(err error) bool {
	if err != nil {
		switch err.(type) {
		case ScopedWriteCloserScopeChangedError:
			if s.currentReceiptIsTarget {
				return true
			}
			s.currentReceiptIsTarget = s.targetReceipt == err.Error()
		case ScopedWriteCloserScopeMismatchError:
			return false
		default:
			log.Fatal(err)
		}
	}
	return false
}

func main() {
	if receiptNameRegExpError != nil {
		log.Fatal(receiptNameRegExpError)
	}
	file, err := os.Open("receipts")
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	if !scanner.Scan() {
		log.Fatal("No receipt found")
	}
	var line = scanner.Text()
	var output StringWriteCloser
	output, err = makeOutput(line)
	if err != nil {
		log.Fatal(err)
	}
	defer output.Close()
	if len(os.Args) < 2 {
		log.Fatal("No target receipt")
	}
	state := &AppState{
		targetReceipt:          os.Args[1],
		currentReceiptIsTarget: false,
	}
	_, err = output.Write(line)
	state.isCookingDone(err)

	for scanner.Scan() {
		_, err := output.Write(line)
		if state.isCookingDone(err) {
			break
		}
	}

	if err := scanner.Err(); err != nil {
		log.Fatal(err)
	}
}
