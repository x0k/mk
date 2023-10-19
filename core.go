package main

import "bufio"

type RecipeLinesCollector interface {
	CollectLines(scanner *bufio.Scanner) (bool, error)
	GetLines() string
}

type RecipeLinesPrinter interface {
	Print(lines string) error
}
