package main

import (
	"errors"
	"regexp"
)

type LinesPrinter interface {
	Print(lines string) error
}

const DEFAULT_TARGET_SEGMENT = "all"

type SegmentsScannerState int

const (
	SEGMENT_NOT_DEFINED SegmentsScannerState = iota
	SEGMENT_STARTS
	SEGMENT_CONTINUED
)

var SEGMENT_NAME_REG_EXP = regexp.MustCompile(`^([A-z][0-9A-z_-]*):(.*)$`)
var SEGMENT_INDENT_REG_EXP = regexp.MustCompile(`^([ \t]+)`)
var ErrSegmentNotFound = errors.New("Segment not found")
