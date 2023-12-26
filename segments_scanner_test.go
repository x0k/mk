package main

import (
	"strings"
	"testing"
)

type segmentData struct {
	state   SegmentsScannerState
	name    string
	targets string
	text    string
}

func TestCompactScan(t *testing.T) {
	scanner := NewSegmentsScanner(strings.NewReader(`foo: bar baz
	foo-content
common-content
bar:
	bar-content
all:`))
	segments := []segmentData{
		{SEGMENT_CONTINUED, "foo", " bar baz", "foo-content\n"},
		{SEGMENT_NOT_DEFINED, "", "", "common-content\n"},
		{SEGMENT_CONTINUED, "bar", "", "bar-content\n"},
		{SEGMENT_STARTS, "all", "", ""},
	}
	for i := 0; scanner.Scan(); i++ {
		if i >= len(segments) {
			t.Errorf("Too many segments")
			return
		}
		expectedSegment := segments[i]
		state, segment, targets := scanner.State()
		text := scanner.Text()
		if state != expectedSegment.state {
			t.Errorf("Expected state %d, got %d for segment %q", expectedSegment.state, state, expectedSegment.name)
		}
		if segment != expectedSegment.name {
			t.Errorf("Expected segment %q, got %q", expectedSegment.name, segment)
		}
		if targets != expectedSegment.targets {
			t.Errorf("Expected targets %q, got %q", expectedSegment.targets, targets)
		}
		if text != expectedSegment.text {
			t.Errorf("Expected text %q, got %q", expectedSegment.text, text)
		}
	}
}

func TestSpacesScan(t *testing.T) {
	scanner := NewSegmentsScanner(strings.NewReader(`#!/bin/bash -xe

foo:
	line-1
	line-2

all:

`))
	segments := []segmentData{
		{SEGMENT_NOT_DEFINED, "", "", "#!/bin/bash -xe\n\n"},
		{SEGMENT_CONTINUED, "foo", "", "line-1\nline-2\n"},
		{SEGMENT_NOT_DEFINED, "", "", "\n"},
		{SEGMENT_STARTS, "all", "", ""},
		{SEGMENT_NOT_DEFINED, "", "", "\n"},
	}
	for i := 0; scanner.Scan(); i++ {
		if i >= len(segments) {
			t.Errorf("Too many segments")
			return
		}
		expectedSegment := segments[i]
		state, segment, targets := scanner.State()
		text := scanner.Text()
		if state != expectedSegment.state {
			t.Errorf("Expected state %d, got %d for segment %q", expectedSegment.state, state, expectedSegment.name)
		}
		if segment != expectedSegment.name {
			t.Errorf("Expected segment %q, got %q", expectedSegment.name, segment)
		}
		if targets != expectedSegment.targets {
			t.Errorf("Expected targets %q, got %q", expectedSegment.targets, targets)
		}
		if text != expectedSegment.text {
			t.Errorf("Expected text %q, got %q", expectedSegment.text, text)
		}
	}
}
