package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"os"
	"regexp"
	"strings"
)

type StringWriteCloser interface {
	Write(line string) (int, error)
	io.Closer
}

type StringWriter interface {
	Write(line string) error
}

type StdWriteCloser struct{}

func (StdWriteCloser) Write(line string) (int, error) {
	return fmt.Println(line)
}

func (StdWriteCloser) Close() error {
	return nil
}

const (
	rootScope = ":root:"
)

var receiptNameRegExp, receiptNameRegExpError = regexp.Compile(`^[A-Za-z][0-9A-Za-z\t _-]*:$`)
var scopeIdentRegExp, scopeIdentRegExpError = regexp.Compile(`^([ \t]+)`)

type ScopedStringWriter struct {
	targetScope             string
	currentScope            string
	shouldDefineScopeIndent bool
	currentScopeIndentation string
	lines                   []string
}

type ScopedStringWriterDoneError struct{}

func (e ScopedStringWriterDoneError) Error() string {
	return "done"
}

func (r *ScopedStringWriter) setScope(line string) {
	r.currentScope = line[:len(line)-1]
	r.shouldDefineScopeIndent = true
}

func (r *ScopedStringWriter) switchScope(line string) error {
	if r.currentScope == r.targetScope {
		return &ScopedStringWriterDoneError{}
	}
	if receiptNameRegExp.MatchString(line) {
		r.setScope(line)
	} else {
		r.currentScope = rootScope
	}
	return nil
}

func (r *ScopedStringWriter) appendScopedLine(line string) {
	if r.targetScope == r.currentScope {
		r.lines = append(r.lines, strings.TrimPrefix(line, r.currentScopeIndentation))
	}
}

func (r *ScopedStringWriter) Write(line string) error {
	if r.currentScope == rootScope {
		if receiptNameRegExp.MatchString(line) {
			r.setScope(line)
		} else {
			r.lines = append(r.lines, line)
		}
	} else {
		if r.shouldDefineScopeIndent {
			r.shouldDefineScopeIndent = false
			matches := scopeIdentRegExp.FindStringSubmatch(line)
			if len(matches) == 2 {
				r.currentScopeIndentation = matches[1]
				r.appendScopedLine(line)
			} else if err := r.switchScope(line); err != nil {
				return err
			}
		} else {
			if strings.HasPrefix(line, r.currentScopeIndentation) {
				r.appendScopedLine(line)
			} else if err := r.switchScope(line); err != nil {
				return err
			}
		}
	}
	return nil
}

func collectScopeStrings(scanner *bufio.Scanner, writer *ScopedStringWriter) (bool, error) {
	for scanner.Scan() {
		if err := (*writer).Write(scanner.Text()); err != nil {
			switch err.(type) {
			case ScopedStringWriterDoneError:
				return true, nil
			default:
				return false, err
			}
		}
	}
	return false, scanner.Err()
}

func main() {
	if len(os.Args) < 2 {
		log.Fatal("No receipt provided")
	}
	file, err := os.Open("receipts")
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	linesCollector := &ScopedStringWriter{
		targetScope:  os.Args[1],
		currentScope: rootScope,
	}
	isReceiptFounded, err := collectScopeStrings(scanner, linesCollector)
	if err != nil {
		log.Fatal(err)
	}
	if !isReceiptFounded {
		log.Fatal("No receipt found")
	}
	out := &StdWriteCloser{}
	defer out.Close()
	for i := 0; i < len(linesCollector.lines); i++ {
		if _, err := out.Write(linesCollector.lines[i] + "\n"); err != nil {
			log.Fatal(err)
		}
	}
}
