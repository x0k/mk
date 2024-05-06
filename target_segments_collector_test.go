package main

import (
	"errors"
	"strings"
	"testing"
)

func TestTargetSegmentsCollector(t *testing.T) {
	tests := []struct {
		name          string
		target        string
		content       string
		expected      string
		expectedError error
	}{
		{
			name:   "Specific target segment",
			target: "baz",
			content: `#!/bin/bash -xe
foo: baz
	foo
bar
baz:
	baz`,
			expected: `#!/bin/bash -xe
foo
bar
baz
`,
		},
		{
			name:   "Default target segment",
			target: DEFAULT_TARGET_SEGMENT,
			content: `#!/bin/bash -xe
foo: mk baz
	foo
xxx: !
	excluded
bar
mk:
baz`,
			expected: `#!/bin/bash -xe
foo
bar
`,
		},
		{
			name:   "Segment not found",
			target: "bar",
			content: `foo: baz
	foo
bar
baz:
	baz`,
			expectedError: ErrSegmentNotFound,
		},
		{
			name:   "Segment target as prefix",
			target: "build.foo",
			content: `prepare: build
	prepare
build.bar:
	bar
build.foo:
	foo`,
			expected: `prepare
foo
`,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			b := strings.Builder{}
			collector := NewTargetSegmentsCollector(tt.target)
			scanner := NewSegmentsScanner(strings.NewReader(tt.content))
			err := collector.Collect(scanner, &b)
			if err != tt.expectedError && !errors.Is(err, tt.expectedError) {
				t.Errorf("Expected error %q, got %q", tt.expectedError, err)
			}
			if tt.expectedError == nil && b.String() != tt.expected {
				t.Errorf("Expected string %q, got %q", tt.expected, b.String())
			}
		})
	}
}
