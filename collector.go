package main

import (
	"regexp"
	"strings"
)

const (
	rootScope = ":root:"
)

var receiptNameRegExp, _ = regexp.Compile(`^[A-Za-z][0-9A-Za-z\t _-]*:$`)
var scopeIdentRegExp, _ = regexp.Compile(`^([ \t]+)`)

type receiptLinesCollector struct {
	targetScope             string
	currentScope            string
	shouldDefineScopeIndent bool
	currentScopeIndentation string
	lines                   []string
}

func NewReceiptLinesCollector(targetScope string) ReceiptLinesCollector {
	return &receiptLinesCollector{
		targetScope:  targetScope,
		currentScope: rootScope,
	}
}

func (r *receiptLinesCollector) setScope(line string) {
	r.currentScope = line[:len(line)-1]
	r.shouldDefineScopeIndent = true
}

func (r *receiptLinesCollector) switchScope(line string) error {
	if r.currentScope == r.targetScope {
		return RLCDoneError
	}
	if receiptNameRegExp.MatchString(line) {
		r.setScope(line)
	} else {
		r.currentScope = rootScope
	}
	return nil
}

func (r *receiptLinesCollector) appendLine(line string) {
	r.lines = append(r.lines, line)
}

func (r *receiptLinesCollector) appendScopedLine(line string) {
	if r.targetScope == r.currentScope {
		r.appendLine(strings.TrimPrefix(line, r.currentScopeIndentation))
	}
}

func (r *receiptLinesCollector) WriteString(line string) (int, error) {
	if r.currentScope == rootScope {
		if receiptNameRegExp.MatchString(line) {
			r.setScope(line)
		} else {
			r.appendLine(line)
		}
	} else {
		if r.shouldDefineScopeIndent {
			r.shouldDefineScopeIndent = false
			matches := scopeIdentRegExp.FindStringSubmatch(line)
			if len(matches) == 2 {
				r.currentScopeIndentation = matches[1]
				r.appendScopedLine(line)
			} else if err := r.switchScope(line); err != nil {
				return 0, err
			}
		} else {
			if strings.HasPrefix(line, r.currentScopeIndentation) {
				r.appendScopedLine(line)
			} else if err := r.switchScope(line); err != nil {
				return 0, err
			}
		}
	}
	return len(line), nil
}

func (r *receiptLinesCollector) GetLines() []string {
	return r.lines
}

func (r *receiptLinesCollector) IsInTargetReceipt() bool {
	return r.currentScope == r.targetScope
}
