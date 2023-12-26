package main

import (
	"bufio"
	"io"
	"strings"
)

type segmentsScanner struct {
	scanner            *bufio.Scanner
	currentState       SegmentsScannerState
	currentSegment     string
	currentTargets     string
	lastState          SegmentsScannerState
	lastSegment        string
	lastTargets        string
	lastSegmentContent string
	segmentIndentation string
	segmentBuilder     strings.Builder
	done               bool
}

func NewSegmentsScanner(reader io.Reader) *segmentsScanner {
	return &segmentsScanner{
		scanner: bufio.NewScanner(reader),
	}
}

func (r *segmentsScanner) setState(state SegmentsScannerState, segment string, targets string) {
	r.lastState = r.currentState
	r.lastSegment = r.currentSegment
	r.lastTargets = r.currentTargets
	r.lastSegmentContent = r.segmentBuilder.String()
	r.segmentBuilder.Reset()
	r.currentState = state
	r.currentSegment = segment
	r.currentTargets = targets
}

func (r *segmentsScanner) setToken(line string) {
	r.segmentBuilder.WriteString(line)
	r.segmentBuilder.WriteByte('\n')
}

func (r *segmentsScanner) setSegmentToken(line string) {
	r.setToken(strings.TrimPrefix(line, r.segmentIndentation))
}

func (r *segmentsScanner) tryStartSegment(line string) bool {
	matched := SEGMENT_NAME_REG_EXP.FindStringSubmatch(line)
	if matched == nil {
		return false
	}
	r.setState(SEGMENT_STARTS, matched[1], matched[2])
	return true
}

func (r *segmentsScanner) finishSegment(line string) {
	if !r.tryStartSegment(line) {
		r.setState(SEGMENT_NOT_DEFINED, "", "")
		r.setToken(line)
	}
}

func (r *segmentsScanner) processLine(line string) bool {
	switch r.currentState {
	case SEGMENT_NOT_DEFINED:
		if r.tryStartSegment(line) {
			return true
		} else {
			r.setToken(line)
		}
	case SEGMENT_STARTS:
		matches := SEGMENT_INDENT_REG_EXP.FindStringSubmatch(line)
		if matches != nil {
			r.segmentIndentation = matches[1]
			r.currentState = SEGMENT_CONTINUED
			r.setSegmentToken(line)
		} else {
			r.finishSegment(line)
			return true
		}
	case SEGMENT_CONTINUED:
		if strings.HasPrefix(line, r.segmentIndentation) {
			r.setSegmentToken(line)
		} else {
			r.finishSegment(line)
			return true
		}
	}
	return false
}

func (r *segmentsScanner) State() (state SegmentsScannerState, segment string, targets string) {
	state = r.lastState
	segment = r.lastSegment
	targets = r.lastTargets
	return
}

func (r *segmentsScanner) Scan() bool {
	if r.done {
		return false
	}
	for r.scanner.Scan() {
		if r.processLine(r.scanner.Text()) {
			return true
		}
	}
	r.done = true
	// To process last segment
	if r.segmentBuilder.Len() > 0 {
		r.setState(SEGMENT_NOT_DEFINED, "", "")
		return true
	}
	return false
}

func (r *segmentsScanner) Err() error {
	return r.scanner.Err()
}

func (r *segmentsScanner) Text() string {
	return r.lastSegmentContent
}
