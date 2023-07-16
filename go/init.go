package specimen

import (
	"log"

	"github.com/ditrit/specimen/go/syaml"
)

var _ = func() int {
	log.SetFlags(0)
	return 0
}()

var syc = syaml.NewSyaml(true, 90)
