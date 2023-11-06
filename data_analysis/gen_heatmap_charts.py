# To install the required dependencies, run the following command:
#   pip install pandas matplotlib seaborn
# To generate the charts, run the following command in the same directory as
# this file:
#   python gen_heatmap_charts.py
#
# The assumption is that the CSV file has been generated using the following
# command:
# go run tools/trace_producer_simu/main.go -batch-size=10,100,500,1000,2500,5000,10000 --max-batches-per-stream=10,100,500,1000,2500,5000,10000 <your-json-files>

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from matplotlib.colors import PowerNorm

# Load the dataset
new_data_path = '../compression-efficiency-gain.csv'  # Replace with your CSV file path
new_data = pd.read_csv(new_data_path)

# Data cleaning: Remove leading and trailing spaces from column names
new_data.columns = new_data.columns.str.strip()

# Calculate the compressed sizes in MB for both OTLP and OTel Arrow
new_data['OTLP_Compressed_MB'] = new_data['OTLP batch compressed size'] / 1_048_576
new_data['OTel_Arrow_Compressed_MB'] = new_data['OTel Arrow batch compressed size'] / 1_048_576

# Calculate the percentage of improvement of OTel Arrow over OTLP
new_data['Improvement_Percentage'] = ((new_data['OTLP_Compressed_MB'] - new_data['OTel_Arrow_Compressed_MB']) /
                                      new_data['OTLP_Compressed_MB']) * 100

# Group by 'Batch size' and 'Max batches per stream' and calculate the mean 'Improvement_Percentage'
grouped_improvement = new_data.groupby(['Batch size', 'Max batches per stream']).agg({
    'Improvement_Percentage': 'mean'
}).reset_index()

# Pivot the data for heatmap plotting
pivot_improvement = grouped_improvement.pivot(index='Batch size', columns='Max batches per stream', values='Improvement_Percentage')

# Plot the heatmap for the average percentage of compressed size improvement with an inverted color scale
plt.figure(figsize=(12, 9))
sns.heatmap(pivot_improvement, annot=True, fmt=".1f", cmap="YlGnBu_r", linewidths=.5)
plt.title('Average Percentage of Compressed Size Improvement of OTel Arrow over OTLP')
plt.xlabel('Max Batches Per Stream')
plt.ylabel('Batch Size')
# Add percentage signs to the annotations
for text in plt.gca().texts:
    text.set_text(text.get_text() + "%")
#plt.show()
plt.savefig('average_improvement_heatmap.png', bbox_inches='tight')

# Side-by-side Heatmaps for Compressed Size
# Define the function to create a side-by-side heatmap for the given dataset using a blue-to-yellow color scale without overlapping color bar
def create_side_by_side_heatmap(data):
    # Group by 'Batch size' and 'Max batches per stream' and sum the compressed sizes
    grouped_data = data.groupby(['Batch size', 'Max batches per stream']).agg({
        'OTLP_Compressed_MB': 'sum',
        'OTel_Arrow_Compressed_MB': 'sum'
    }).reset_index()

    # Pivot the data for heatmap plotting
    pivot_OTLP = grouped_data.pivot(index='Batch size', columns='Max batches per stream', values='OTLP_Compressed_MB')
    pivot_OTel_Arrow = grouped_data.pivot(index='Batch size', columns='Max batches per stream', values='OTel_Arrow_Compressed_MB')

    # Define the color map and normalization
    cmap = sns.color_palette("YlGnBu", as_cmap=True)
    norm = PowerNorm(gamma=0.5, vmin=min(pivot_OTLP.min().min(), pivot_OTel_Arrow.min().min()),
                     vmax=max(pivot_OTLP.max().max(), pivot_OTel_Arrow.max().max()))

    # Create subplots
    fig, axn = plt.subplots(1, 2, sharex=True, sharey=True, figsize=(18, 8))
    cbar_ax = fig.add_axes([.91, .3, .03, .4])

    # OTLP heatmap
    sns.heatmap(pivot_OTLP, ax=axn[0], cbar=True, cbar_ax=cbar_ax, annot=True, fmt=".1f",
                linewidths=.5, cmap=cmap, norm=norm)
    axn[0].set_title('OTLP Compressed Size (MB)')
    axn[0].set_xlabel('Max Batches Per Stream')
    axn[0].set_ylabel('Batch Size')

    # OTel Arrow heatmap
    sns.heatmap(pivot_OTel_Arrow, ax=axn[1], cbar=False, annot=True, fmt=".1f",
                linewidths=.5, cmap=cmap, norm=norm)
    axn[1].set_title('OTel Arrow Compressed Size (MB)')
    axn[1].set_xlabel('Max Batches Per Stream')
    axn[1].set_ylabel('')

    # Adjust layout
    plt.tight_layout(rect=[0, 0, .9, 1])
    plt.savefig('side_by_side_heatmap.png', bbox_inches='tight')
    # plt.show()

# Call the function with the data
create_side_by_side_heatmap(new_data)
