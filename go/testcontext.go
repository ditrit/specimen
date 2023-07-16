package specimen

import (
	"strings"

	"github.com/go-test/deep"
)

// Fail marks the slab as failed and saves the given information, if provided. It can be called multiple times for a single slab. All saved information will be reported.
func (s *S) Fail(info string) {
	s.status = Failed
	if len(info) > 0 {
		s.failInfo = append(s.failInfo, info)
	}
}

// Abort marks the slab as aborted and saves the given information, if provided.
func (s *S) Abort(info string) {
	s.status = Aborted
	if len(info) > 0 {
		s.failInfo = append(s.failInfo, info)
	}
	panic(nil)
}

// ExpectEqual tests if the two given values are equal data structures
func (s *S) ExpectEqual(value, wanted interface{}, context string) {
	if diff := deep.Equal(value, wanted); diff != nil {
		if len(context) > 0 {
			context = "(" + context + "): "
		}
		s.Fail(context + strings.Join(diff, ", "))
	}
}
