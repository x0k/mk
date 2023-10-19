package main

import (
	"bufio"
	"regexp"
	"strings"
)

const (
	SEGMENT_NOT_DEFINED = 0
	SEGMENT_STARTS      = 1
	SEGMENT_CONTINUED   = 2
)

var SEGMENT_NAME_REG_EXP = regexp.MustCompile(`^[A-Za-z][0-9A-Za-z\t _-]*:$`)
var SEGMENT_INDENT_REG_EXP = regexp.MustCompile(`^([ \t]+)`)

type segmentLinesCollector struct {
	state              int
	targetSegment      string
	isSegmentFound     bool
	segmentIndentation string
	lines              string
}

func NewSegmentLinesCollector(targetSegment string) LinesCollector {
	return &segmentLinesCollector{
		targetSegment: targetSegment,
	}
}

func (r *segmentLinesCollector) appendLine(line string) {
	r.lines = r.lines + line + "\n"
}

func (r *segmentLinesCollector) appendSegmentLine(line string) {
	if r.isSegmentFound {
		r.appendLine(strings.TrimPrefix(line, r.segmentIndentation))
	}
}

func (r *segmentLinesCollector) startSegment(line string) {
	r.isSegmentFound = r.targetSegment == line[:len(line)-1]
	r.state = SEGMENT_STARTS
}

func (r *segmentLinesCollector) handleSegmentChange(line string) {
	if SEGMENT_NAME_REG_EXP.MatchString(line) {
		r.startSegment(line)
	} else {
		r.appendLine(line)
		r.state = SEGMENT_NOT_DEFINED
	}
}

func (r *segmentLinesCollector) collectLine(line string) bool {
	switch r.state {
	case SEGMENT_NOT_DEFINED:
		if SEGMENT_NAME_REG_EXP.MatchString(line) {
			r.startSegment(line)
		} else {
			r.appendLine(line)
		}
	case SEGMENT_STARTS:
		matches := SEGMENT_INDENT_REG_EXP.FindStringSubmatch(line)
		if matches != nil {
			r.segmentIndentation = matches[1]
			r.appendSegmentLine(line)
			r.state = SEGMENT_CONTINUED
		} else if r.isSegmentFound {
			return true
		} else {
			r.handleSegmentChange(line)
		}
	case SEGMENT_CONTINUED:
		if strings.HasPrefix(line, r.segmentIndentation) {
			r.appendSegmentLine(line)
		} else if r.isSegmentFound {
			return true
		} else {
			r.handleSegmentChange(line)
		}
	}
	return false
}

func (r *segmentLinesCollector) CollectLines(scanner *bufio.Scanner) (bool, error) {
	for scanner.Scan() {
		if targetSegmentEnded := r.collectLine(scanner.Text()); targetSegmentEnded {
			return targetSegmentEnded, nil
		}
	}
	if err := scanner.Err(); err != nil {
		return false, err
	}
	return r.isSegmentFound, nil
}

func (r *segmentLinesCollector) GetLines() string {
	return r.lines
}
