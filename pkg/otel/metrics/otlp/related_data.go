/*
 * Copyright The OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package otlp

import (
	colarspb "github.com/f5/otel-arrow-adapter/api/experimental/arrow/v1"
	"github.com/f5/otel-arrow-adapter/pkg/otel"
	"github.com/f5/otel-arrow-adapter/pkg/otel/common/otlp"
	"github.com/f5/otel-arrow-adapter/pkg/record_message"
	"github.com/f5/otel-arrow-adapter/pkg/werror"
)

type (
	RelatedData struct {
		MetricID uint16

		// Attributes stores
		ResAttrMapStore        *otlp.Attributes16Store
		ScopeAttrMapStore      *otlp.Attributes16Store
		SumIntAttrsStore       *otlp.Attributes32Store
		SumDoubleAttrsStore    *otlp.Attributes32Store
		SummaryAttrsStore      *otlp.Attributes32Store
		GaugeIntAttrsStore     *otlp.Attributes32Store
		GaugeDoubleAttrsStore  *otlp.Attributes32Store
		HistogramAttrsStore    *otlp.Attributes32Store
		ExpHistogramAttrsStore *otlp.Attributes32Store

		// Metric stores
		SumIntDataPointsStore      *SumIntDataPointsStore
		SumDoubleDataPointsStore   *SumDoubleDataPointsStore
		GaugeIntDataPointsStore    *GaugeIntDataPointsStore
		GaugeDoubleDataPointsStore *GaugeDoubleDataPointsStore
		SummaryDataPointsStore     *SummaryDataPointsStore
		HistogramDataPointsStore   *HistogramDataPointsStore
		EHistogramDataPointsStore  *EHistogramDataPointsStore
	}
)

func NewRelatedData() *RelatedData {
	return &RelatedData{
		ResAttrMapStore:        otlp.NewAttributes16Store(),
		ScopeAttrMapStore:      otlp.NewAttributes16Store(),
		SumIntAttrsStore:       otlp.NewAttributes32Store(),
		SumDoubleAttrsStore:    otlp.NewAttributes32Store(),
		SummaryAttrsStore:      otlp.NewAttributes32Store(),
		GaugeIntAttrsStore:     otlp.NewAttributes32Store(),
		GaugeDoubleAttrsStore:  otlp.NewAttributes32Store(),
		HistogramAttrsStore:    otlp.NewAttributes32Store(),
		ExpHistogramAttrsStore: otlp.NewAttributes32Store(),

		SumIntDataPointsStore:      NewSumIntDataPointsStore(),
		SumDoubleDataPointsStore:   NewSumDoubleDataPointsStore(),
		GaugeIntDataPointsStore:    NewGaugeIntDataPointsStore(),
		GaugeDoubleDataPointsStore: NewGaugeDoubleDataPointsStore(),
		SummaryDataPointsStore:     NewSummaryDataPointsStore(),
		HistogramDataPointsStore:   NewHistogramDataPointsStore(),
		EHistogramDataPointsStore:  NewEHistogramDataPointsStore(),
	}
}

func (r *RelatedData) MetricIDFromDelta(delta uint16) uint16 {
	r.MetricID += delta
	return r.MetricID
}

func RelatedDataFrom(records []*record_message.RecordMessage) (relatedData *RelatedData, metricsRecord *record_message.RecordMessage, err error) {
	var sumIntDPRec *record_message.RecordMessage
	var sumDoubleDPRec *record_message.RecordMessage
	var gaugeIntDPRec *record_message.RecordMessage
	var gaugeDoubleDPRec *record_message.RecordMessage
	var summaryDPRec *record_message.RecordMessage
	var histogramDPRec *record_message.RecordMessage
	var expHistogramDPRec *record_message.RecordMessage

	relatedData = NewRelatedData()

	for _, record := range records {
		switch record.PayloadType() {
		case colarspb.OtlpArrowPayloadType_RESOURCE_ATTRS:
			err = otlp.Attributes16StoreFrom(record.Record(), relatedData.ResAttrMapStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_SCOPE_ATTRS:
			err = otlp.Attributes16StoreFrom(record.Record(), relatedData.ScopeAttrMapStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_INT_SUM_ATTRS:
			err = otlp.Attributes32StoreFrom(record.Record(), relatedData.SumIntAttrsStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_DOUBLE_SUM_ATTRS:
			err = otlp.Attributes32StoreFrom(record.Record(), relatedData.SumDoubleAttrsStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_INT_GAUGE_ATTRS:
			err = otlp.Attributes32StoreFrom(record.Record(), relatedData.GaugeIntAttrsStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_DOUBLE_GAUGE_ATTRS:
			err = otlp.Attributes32StoreFrom(record.Record(), relatedData.GaugeDoubleAttrsStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_SUMMARY_ATTRS:
			err = otlp.Attributes32StoreFrom(record.Record(), relatedData.SummaryAttrsStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_HISTOGRAM_ATTRS:
			err = otlp.Attributes32StoreFrom(record.Record(), relatedData.HistogramAttrsStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_EXP_HISTOGRAM_ATTRS:
			err = otlp.Attributes32StoreFrom(record.Record(), relatedData.ExpHistogramAttrsStore)
			if err != nil {
				return nil, nil, werror.Wrap(err)
			}
		case colarspb.OtlpArrowPayloadType_INT_SUM:
			if sumIntDPRec != nil {
				return nil, nil, werror.Wrap(otel.ErrDuplicatePayloadType)
			}
			sumIntDPRec = record
		case colarspb.OtlpArrowPayloadType_DOUBLE_SUM:
			if sumDoubleDPRec != nil {
				return nil, nil, werror.Wrap(otel.ErrDuplicatePayloadType)
			}
			sumDoubleDPRec = record
		case colarspb.OtlpArrowPayloadType_INT_GAUGE:
			if gaugeIntDPRec != nil {
				return nil, nil, werror.Wrap(otel.ErrDuplicatePayloadType)
			}
			gaugeIntDPRec = record
		case colarspb.OtlpArrowPayloadType_DOUBLE_GAUGE:
			if gaugeDoubleDPRec != nil {
				return nil, nil, werror.Wrap(otel.ErrDuplicatePayloadType)
			}
			gaugeDoubleDPRec = record
		case colarspb.OtlpArrowPayloadType_SUMMARIES:
			if summaryDPRec != nil {
				return nil, nil, werror.Wrap(otel.ErrDuplicatePayloadType)
			}
			summaryDPRec = record
		case colarspb.OtlpArrowPayloadType_HISTOGRAMS:
			if histogramDPRec != nil {
				return nil, nil, werror.Wrap(otel.ErrDuplicatePayloadType)
			}
			histogramDPRec = record
		case colarspb.OtlpArrowPayloadType_EXP_HISTOGRAMS:
			if expHistogramDPRec != nil {
				return nil, nil, werror.Wrap(otel.ErrDuplicatePayloadType)
			}
			expHistogramDPRec = record
		case colarspb.OtlpArrowPayloadType_METRICS:
			if metricsRecord != nil {
				return nil, nil, werror.Wrap(otel.ErrDuplicatePayloadType)
			}
			metricsRecord = record
		default:
			return nil, nil, werror.Wrap(otel.UnknownPayloadType)
		}
	}

	if sumIntDPRec != nil {
		relatedData.SumIntDataPointsStore, err = SumIntStoreFrom(sumIntDPRec.Record(), relatedData.SumIntAttrsStore)
		if err != nil {
			return nil, nil, werror.Wrap(err)
		}
	}

	if sumDoubleDPRec != nil {
		relatedData.SumDoubleDataPointsStore, err = SumDoubleStoreFrom(sumDoubleDPRec.Record(), relatedData.SumDoubleAttrsStore)
		if err != nil {
			return nil, nil, werror.Wrap(err)
		}
	}

	if gaugeIntDPRec != nil {
		relatedData.GaugeIntDataPointsStore, err = GaugeIntStoreFrom(gaugeIntDPRec.Record(), relatedData.GaugeIntAttrsStore)
		if err != nil {
			return nil, nil, werror.Wrap(err)
		}
	}

	if gaugeDoubleDPRec != nil {
		relatedData.GaugeDoubleDataPointsStore, err = GaugeDoubleStoreFrom(gaugeDoubleDPRec.Record(), relatedData.GaugeDoubleAttrsStore)
		if err != nil {
			return nil, nil, werror.Wrap(err)
		}
	}

	if summaryDPRec != nil {
		relatedData.SummaryDataPointsStore, err = SummaryDataPointsStoreFrom(summaryDPRec.Record(), relatedData.SummaryAttrsStore)
		if err != nil {
			return nil, nil, werror.Wrap(err)
		}
	}

	if histogramDPRec != nil {
		relatedData.HistogramDataPointsStore, err = HistogramDataPointsStoreFrom(histogramDPRec.Record(), relatedData.HistogramAttrsStore)
		if err != nil {
			return nil, nil, werror.Wrap(err)
		}
	}

	if expHistogramDPRec != nil {
		relatedData.EHistogramDataPointsStore, err = EHistogramDataPointsStoreFrom(expHistogramDPRec.Record(), relatedData.ExpHistogramAttrsStore)
		if err != nil {
			return nil, nil, werror.Wrap(err)
		}
	}

	return
}
