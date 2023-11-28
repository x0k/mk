package main

import (
	"bufio"
	"strings"
)

type allLinesCollector struct {
	state              int
	segmentIndentation string
	lines              string
}

func NewAllLinesCollector() LinesCollector {
	return &allLinesCollector{}
}

func (r *allLinesCollector) appendLine(line string) {
	r.lines = r.lines + line + "\n"
}

func (r *allLinesCollector) appendSegmentLine(line string) {
	r.appendLine(strings.TrimPrefix(line, r.segmentIndentation))
}

func (r *allLinesCollector) finishSegment(line string) {
	if SEGMENT_NAME_REG_EXP.MatchString(line) {
		r.state = SEGMENT_STARTS
	} else {
		r.appendLine(line)
		r.state = SEGMENT_NOT_DEFINED
	}
}

func (r *allLinesCollector) collectLine(line string) {
	switch r.state {
	case SEGMENT_NOT_DEFINED:
		if SEGMENT_NAME_REG_EXP.MatchString(line) {
			r.state = SEGMENT_STARTS
		} else {
			r.appendLine(line)
		}
	case SEGMENT_STARTS:
		matches := SEGMENT_INDENT_REG_EXP.FindStringSubmatch(line)
		if matches != nil {
			r.segmentIndentation = matches[1]
			r.appendSegmentLine(line)
			r.state = SEGMENT_CONTINUED
		} else {
			r.finishSegment(line)
		}
	case SEGMENT_CONTINUED:
		if strings.HasPrefix(line, r.segmentIndentation) {
			r.appendSegmentLine(line)
		} else {
			r.finishSegment(line)
		}
	}
}

func (r *allLinesCollector) CollectLines(scanner *bufio.Scanner) (bool, error) {
	for scanner.Scan() {
		r.collectLine(scanner.Text())
	}
	if err := scanner.Err(); err != nil {
		return false, err
	}
	return true, nil
}

func (r *allLinesCollector) GetLines() string {
	return r.lines
}
