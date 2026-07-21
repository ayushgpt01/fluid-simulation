1. Add inputs to pick up and release water with mouse
2. GPU upgrade.

Main loop calls in order -

- externalForcesKernel (GPU)
- RunSpatial (CPU)
- densityKernel (GPU)
- pressureKernel (GPU)
- viscosityKernel (GPU)
- updatePositionKernel (GPU)

Then Inside RunSpatial -

- gpuSort.Run(SpatialIndices, SpatialKeys, (uint)(SpatialKeys.count - 1))
- spatialOffsetsCalc.Run(true, SpatialKeys, SpatialOffsets)

Then inside gpuSort -

- Sets buffers for sortedItemsBuffer, sortedValuesBuffer, countsBuffer, itemsBuffer, keysBuffer
- ClearCountsKernel (GPU)
- CountKernel (GPU)
- scan.Run(countsBuffer)
- ScatterOutputsKernel (GPU)
- CopyBackKernel (GPU)

And inside spatialOffsetsCalc -

- initKernel (GPU) if needed
- sets buffer for offsets and sortedKeys
- offsetsKernel (GPU)

On the other hand scan.Run does this -

- Sets buffers for groupSumBuffers, elements, sets count
- cs.Dispatch(scanKernel, numGroups, 1, 1); Direct dispatch for a scanKernel (GPU)
- Recursive call for groupSumBuffer
- Again Sets buffers for groupSumBuffers, elements, sets count
- Again runs dispatch for scanKernel (GPU)

This happens every simulation step with all actual work done on GPU
As for init function it sets these buffers up:

```cs
	        SetInitialBufferData(spawnData);

			// Init compute
			ComputeHelper.SetBuffer(compute, positionBuffer, "Positions", externalForcesKernel, updatePositionKernel, reorderKernel, copybackKernel);
			ComputeHelper.SetBuffer(compute, predictedPositionBuffer, "PredictedPositions", externalForcesKernel, spatialHashKernel, densityKernel, pressureKernel, viscosityKernel, reorderKernel, copybackKernel);
			ComputeHelper.SetBuffer(compute, velocityBuffer, "Velocities", externalForcesKernel, pressureKernel, viscosityKernel, updatePositionKernel, reorderKernel, copybackKernel);
			ComputeHelper.SetBuffer(compute, densityBuffer, "Densities", densityKernel, pressureKernel, viscosityKernel);

			ComputeHelper.SetBuffer(compute, spatialHash.SpatialIndices, "SortedIndices", spatialHashKernel, reorderKernel);
			ComputeHelper.SetBuffer(compute, spatialHash.SpatialOffsets, "SpatialOffsets", spatialHashKernel, densityKernel, pressureKernel, viscosityKernel);
			ComputeHelper.SetBuffer(compute, spatialHash.SpatialKeys, "SpatialKeys", spatialHashKernel, densityKernel, pressureKernel, viscosityKernel);

			ComputeHelper.SetBuffer(compute, sortTarget_Position, "SortTarget_Positions", reorderKernel, copybackKernel);
			ComputeHelper.SetBuffer(compute, sortTarget_PredicitedPosition, "SortTarget_PredictedPositions", reorderKernel, copybackKernel);
			ComputeHelper.SetBuffer(compute, sortTarget_Velocity, "SortTarget_Velocities", reorderKernel, copybackKernel);

			compute.SetInt("numParticles", numParticles);
```
