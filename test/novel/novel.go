package novel

type Book struct {
	Title    string
	LeftPage int
	Size     int
}

func (b *Book) TurnPage(count int) {
	b.LeftPage += 2 * count

	if b.LeftPage < 0 {
		b.LeftPage = 0
	} else if b.LeftPage >= b.Size {
		b.LeftPage = b.Size - 1
	}
}

func (b *Book) GetPage() int {
	return b.LeftPage
}
