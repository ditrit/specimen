package orderedstringmap

import "slices"

type OSM struct {
	mapping map[string][]string
	order   []string
}

type Entry struct {
	Key   string
	Value []string
}

func NewOSM() OSM {
	return OSM{mapping: map[string][]string{}}
}

func (o *OSM) Get(key string) []string {
	return o.mapping[key]
}

func (o *OSM) TryGet(key string) *[]string {
	value, found := o.mapping[key]
	if !found {
		return nil
	}
	return &value
}

func (o *OSM) Set(key string, value []string) {
	_, found := o.mapping[key]
	o.mapping[key] = value

	if found {
		// Move the found key to the end of the slice
		shifting := false
		for k := range o.order {
			if o.order[k] == key {
				shifting = true
			}
			if shifting && k+1 < len(o.order) {
				o.order[k] = o.order[k+1]
			}
		}
		o.order[len(o.order)-1] = key
	} else {
		o.order = append(o.order, key)
	}
}

func (o *OSM) Clone() (result OSM) {
	result.mapping = map[string][]string{}
	for key := range o.mapping {
		result.mapping[key] = o.mapping[key]
	}
	result.order = make([]string, len(o.order))
	copy(result.order, o.order)
	return
}

func (o *OSM) Len() int {
	return len(o.order)
}

func (o *OSM) Keys() []string {
	return o.order
}

func (o *OSM) Values() (result [][]string) {
	for _, key := range o.order {
		result = append(result, o.mapping[key])
	}
	return
}

func (o *OSM) Entries() (result []Entry) {
	for _, key := range o.order {
		result = append(result, Entry{Key: key, Value: o.mapping[key]})
	}
	return
}

func (o *OSM) ProductIterator() func() map[string]string {
	reversedKeySlice := make([]string, len(o.order))
	copy(reversedKeySlice, o.order)
	slices.Reverse(reversedKeySlice)
	reversedSizeSlice := make([]int, len(o.order))
	for i, key := range reversedKeySlice {
		reversedSizeSlice[i] = len(o.mapping[key])
	}
	reversedIndexSlice := make([]int, len(o.order))

	combination := map[string]string{}
	for _, key := range reversedKeySlice {
		combination[key] = o.mapping[key][0]
	}

	stopped := false
	first := true

	return func() map[string]string {
		if stopped {
			return nil
		}
		if first {
			first = false
			return combination
		}
		n := -1
		for n < 0 || reversedIndexSlice[n] == 0 {
			n += 1
			if n >= len(reversedIndexSlice) {
				stopped = true
				return nil
			}
			reversedIndexSlice[n] += 1
			reversedIndexSlice[n] %= reversedSizeSlice[n]
			combination[reversedKeySlice[n]] = o.mapping[reversedKeySlice[n]][reversedIndexSlice[n]]
		}

		return combination
	}
}
