package fake

import (
	"github.com/brianvoe/gofakeit/v6"
	"math/rand"
	"time"
)

type DataGenerator struct {
	prevTime    uint64
	currentTime uint64
	id8Bits     []byte
	id16Bits    []byte
}

func NewDataGenerator(currentTime uint64) *DataGenerator {
	return &DataGenerator{
		prevTime:    currentTime,
		currentTime: currentTime,
		id8Bits:     GenId(8),
		id16Bits:    GenId(16),
	}
}

func (dg *DataGenerator) PrevTime() uint64 {
	return dg.prevTime
}

func (dg *DataGenerator) CurrentTime() uint64 {
	return dg.currentTime
}

func (dg *DataGenerator) AdvanceTime(timeDelta time.Duration) {
	dg.prevTime = dg.currentTime
	dg.currentTime += uint64(timeDelta)
}

func (dg *DataGenerator) NextId8Bits() {
	dg.id8Bits = GenId(8)
}

func (dg *DataGenerator) NextId16Bits() {
	dg.id16Bits = GenId(16)
}

func (dg *DataGenerator) Id8Bits() []byte {
	return dg.id8Bits
}

func (dg *DataGenerator) Id16Bits() []byte {
	return dg.id16Bits
}

func (dg *DataGenerator) GenF64Range(min float64, max float64) float64 {
	return min + rand.Float64()*(max-min)
}

func (dg *DataGenerator) GenI64Range(min int64, max int64) int64 {
	return min + int64(rand.Float64()*float64(max-min))
}

func GenId(len uint) []byte {
	return []byte(gofakeit.DigitN(len))
}
