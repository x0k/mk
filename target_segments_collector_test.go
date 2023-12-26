package main

import (
	"strings"
	"testing"
)

func TestCollectSegment(t *testing.T) {
	builder := strings.Builder{}
	target := "baz"
	collector := NewTargetSegmentsCollector(&builder, target)
	scanner := NewSegmentsScanner(strings.NewReader(`#!/bin/bash -xe
foo: baz
	foo
bar
baz:
	baz`))
	expected := `#!/bin/bash -xe
foo
bar
baz
`
	err := collector.Collect(scanner)
	if err != nil {
		t.Errorf("Error during collecting segments %q", err)
	}
	if builder.String() != expected {
		t.Errorf("Expected %q, got %q", expected, builder.String())
	}
}

func TestCollectDefaultSegment(t *testing.T) {
	builder := strings.Builder{}
	collector := NewTargetSegmentsCollector(&builder, DEFAULT_TARGET_SEGMENT)
	scanner := NewSegmentsScanner(strings.NewReader(`#!/bin/bash -xe
foo: baz
	foo
bar
all:
baz`))
	expected := `#!/bin/bash -xe
foo
bar
`
	err := collector.Collect(scanner)
	if err != nil {
		t.Errorf("Error during collecting segments %q", err)
	}
	if builder.String() != expected {
		t.Errorf("Expected %q, got %q", expected, builder.String())
	}
}

func TestSegmentNotFound(t *testing.T) {
	builder := strings.Builder{}
	target := "bar"
	collector := NewTargetSegmentsCollector(&builder, target)
	scanner := NewSegmentsScanner(strings.NewReader(`foo: baz
	foo
bar
baz:
	baz`))
	err := collector.Collect(scanner)
	if err != ErrSegmentNotFound {
		t.Errorf("Expected %q, got %q", ErrSegmentNotFound, err)
	}
}
