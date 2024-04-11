package main

import (
	"bufio"
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

func processFile(l *log.Logger, fileName string, args []string, targetSegment string) bool {
	file, err := os.Open(fileName)
	if err != nil {
		l.Printf("%s: error during opening file: %s", fileName, err)
		return false
	}
	defer file.Close()
	writer, err := makeWriter(file, args)
	if err != nil {
		l.Printf("%s: error during creating writer: %s", fileName, err)
		return false
	}
	err = NewTargetSegmentsCollector(targetSegment).Collect(NewSegmentsScanner(file), writer)
	if err == ErrSegmentNotFound {
		return false
	}
	if err != nil {
		l.Printf("%s: error during collecting segments: %s", fileName, err)
		return false
	}
	if err = writer.Flush(); err != nil {
		l.Printf("%s: error during flushing: %s", fileName, err)
	}
	return true
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
	l := log.New(os.Stderr, "", 0)
	for i := len(dirEntities) - 1; i >= 0; i-- {
		entity := dirEntities[i]
		if entity.IsDir() || !MK_FILE_REG_EXP.MatchString(entity.Name()) {
			continue
		}
		fileFound = true
		if processFile(l, entity.Name(), args, targetSegment) {
			return
		}
	}
	if fileFound {
		log.Fatalf("the segment %q is not found", targetSegment)
	} else {
		log.Fatalf("mkfile is not found, the filename should match: %q", MK_FILE_REG_EXP)
	}
}
