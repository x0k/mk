package main

import (
	"bufio"
	"strings"
)

type segmentLinesCollector struct {
	state                int
	targetSegment        string
	isCollectableSegment bool
	isTargetSegmentFound bool
	segmentIndentation   string
	lines                string
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
	if r.isTargetSegmentFound || r.isCollectableSegment {
		r.appendLine(strings.TrimPrefix(line, r.segmentIndentation))
	}
}

func (r *segmentLinesCollector) tryStartSegment(line string) bool {
	matched := SEGMENT_NAME_REG_EXP.FindStringSubmatch(line)
	if matched == nil {
		return false
	}
	r.state = SEGMENT_STARTS
	if r.targetSegment == matched[1] {
		r.isTargetSegmentFound = true
	} else if r.targetSegment == DEFAULT_TARGET_SEGMENT ||
		(len(matched) > 2 && strings.Contains(matched[2], r.targetSegment)) {
		r.isCollectableSegment = true
	}
	return true
}

func (r *segmentLinesCollector) finishSegment(line string) {
	if r.isTargetSegmentFound {
		r.state = TARGET_SEGMENT_FINISHED
		return
	}
	r.isCollectableSegment = false
	if !r.tryStartSegment(line) {
		r.appendLine(line)
		r.state = SEGMENT_NOT_DEFINED
	}
}

func (r *segmentLinesCollector) collectLine(line string) {
	switch r.state {
	case SEGMENT_NOT_DEFINED:
		if !r.tryStartSegment(line) {
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

func (r *segmentLinesCollector) CollectLines(scanner *bufio.Scanner) (bool, error) {
	for scanner.Scan() {
		r.collectLine(scanner.Text())
		if r.state == TARGET_SEGMENT_FINISHED {
			return true, nil
		}
	}
	if err := scanner.Err(); err != nil {
		return false, err
	}
	return r.isTargetSegmentFound || r.targetSegment == DEFAULT_TARGET_SEGMENT, nil
}

func (r *segmentLinesCollector) Lines() string {
	return r.lines
}
