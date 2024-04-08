package main

import (
	"errors"
	"regexp"
)

type BufferedWriter interface {
	WriteString(string) (int, error)
	Flush() error
}

const DEFAULT_TARGET_SEGMENT = "all"

type SegmentsScannerStateKind int

const (
	SEGMENT_NOT_DEFINED SegmentsScannerStateKind = iota
	SEGMENT_STARTS
	SEGMENT_CONTINUED
)

var MK_FILE_NAMES = []string{"mkfilex", "mkfile", "Mkfilex", "Mkfile"}
var SEGMENT_NAME_REG_EXP = regexp.MustCompile(`^([A-z][0-9A-z_-]*):\s*(.*?)\s*$`)
var SEGMENT_INDENT_REG_EXP = regexp.MustCompile(`^([ \t]+)`)
var ErrSegmentNotFound = errors.New("segment not found")
