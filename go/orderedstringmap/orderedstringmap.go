package orderedstringmap

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
