package main

import "io"

type RecipeLinesCollector interface {
	io.StringWriter
	GetLines() string
	IsRecipeFound() bool
}

type RecipeLinesCollectorDoneError struct{}

var RLCDoneError = &RecipeLinesCollectorDoneError{}

func (e RecipeLinesCollectorDoneError) Error() string {
	return "done"
}

type RecipeLinesPrinter interface {
	Print(lines string) error
}
