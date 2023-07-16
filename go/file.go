package specimen

import "io/ioutil"

// ReadLocalFile reads a file from the file system and returns a specimen.File
func ReadLocalFile(path string) File {
	content, err := ioutil.ReadFile(path)
	if err != nil {
		panic(err)
	}

	return File{
		Path:    path,
		Content: content,
	}
}

// VirtualFile creates a specimen.File from scratch
func VirtualFile(imaginaryPath string, content []byte) File {
	return File{
		Path:    imaginaryPath,
		Content: content,
	}
}

// VirtualFileDedent dedents the given content and creates a specimen.File
func VirtualFileDedent(imaginaryPath string, content []byte) File {
	return VirtualFile(imaginaryPath, dedent(content))
}

// dedent removes common leading **spaces** (not tabs) from the lines of a given text
// note: empty leading newlines are removed. Also, if the first line is non-empty,
// it is excluded from the lines undergoing space trimming.
func dedent(text []byte) []byte {
	// step one: remove leading newlines
	for k, c := range text {
		if c != '\n' {
			text = text[k:]
			break
		}
	}

	// step two: read through the text to know the margin size
	// note only the line which contain some none-whitespace character are
	// taken into account
	margin := len(text)
	currentMargin := 0
	for _, c := range text {
		if c == '\n' {
			currentMargin = 0
		} else if c == ' ' {
			currentMargin += 1
		} else {
			if currentMargin < margin {
				margin = currentMargin
			}
		}
	}

	// step three: compute the size of the buffer required to store the trimmed text
	size := 0
	lineStart := 0
	lineLength := 0
	for k, c := range text {
		if c == '\n' {
			lineLength = k - lineStart
			if lineLength > margin {
				size += lineLength - margin + 1
			} else {
				size += 1
			}
			lineStart = k + 1
		}
	}
	lineLength = len(text) - lineStart
	if lineLength > margin {
		size += lineLength - margin
	}

	// step four: copy the text while removing a margin size from each line
	output := make([]byte, size)

	lineStart = 0
	m := 0
	for k, c := range text {
		if c == '\n' {
			output[m] = c
			m += 1
			lineStart = k + 1
		}
		if k+1-lineStart > margin {
			output[m] = c
			m += 1
		}
	}

	return output
}
