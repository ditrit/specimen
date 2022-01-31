package zoo

import "strings"

type Zoo []string

var zoo Zoo

func AddAnimal(animal string) string {
	zoo = append(zoo, animal)
	return strings.Join(zoo, " ")
}
