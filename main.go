package main

import (
	"bufio"
	"errors"
	"fmt"
	"log"
	"os"
	"strings"
)

func makeWriter(file *os.File, args []string) (BufferedWriter, error) {
	if strings.HasSuffix(file.Name(), "x") {
		return NewCmdWriter(args)
	}
	return bufio.NewWriter(os.Stdout), nil
}

func processFile(fileName string, args []string, targetSegment string) (bool, error) {
	file, err := os.Open(fileName)
	if err != nil {
		return false, fmt.Errorf("error during opening file: %w", err)
	}
	defer file.Close()
	writer, err := makeWriter(file, args)
	if err != nil {
		return false, fmt.Errorf("error during creating writer: %w", err)
	}
	err = NewTargetSegmentsCollector(targetSegment).Collect(NewSegmentsScanner(file), writer)
	if err != nil {
		return false, fmt.Errorf("error during collecting segments: %w", err)
	}
	if err = writer.Flush(); err != nil {
		return true, fmt.Errorf("error during flushing: %w", err)
	}
	return true, nil
}

func main() {
	targetSegment := DEFAULT_TARGET_SEGMENT
	args := []string{}
	if len(os.Args) > 1 {
		targetSegment = os.Args[1]
		args = os.Args[2:]
	}

	dirEntities, err := os.ReadDir(".")
	if err != nil {
		log.Fatalf("error during reading dir: %s", err)
	}

	fileFound := false
	for i := len(dirEntities) - 1; i >= 0; i-- {
		entity := dirEntities[i]
		if entity.IsDir() || !MK_FILE_REG_EXP.MatchString(entity.Name()) {
			continue
		}
		fileFound = true
		isProcessed, err := processFile(entity.Name(), args, targetSegment)
		if isProcessed {
			if err != nil {
				log.Fatalf("%s: %s", entity.Name(), err)
			}
			return
		}
		if errors.Is(err, ErrSegmentNotFound) {
			continue
		}
		if err != nil {
			fmt.Fprintf(os.Stderr, "%s: %s\n", entity.Name(), err)
		}
	}
	if fileFound {
		log.Fatalf("the segment %q is not found", targetSegment)
	} else {
		log.Fatalf("mkfile is not found, the filename should match: %q", MK_FILE_REG_EXP)
	}
}
