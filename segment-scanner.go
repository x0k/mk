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
	segmentIndentation string
	segmentBuilder     strings.Builder
	done               bool
	err                error
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
	r.segmentBuilder.Reset()
	isSegmentUpdated := false
	for !isSegmentUpdated && r.scanner.Scan() {
		isSegmentUpdated = r.processLine(r.scanner.Text())
	}
	if isSegmentUpdated {
		return !r.done
	}
	if err := r.scanner.Err(); err != nil {
		r.err = err
	}
	r.done = true
	return false
}

func (r *segmentsScanner) Err() error {
	return r.err
}

func (r *segmentsScanner) Text() string {
	return r.segmentBuilder.String()
}
