export type Archive = (Solution | null)[]

export type Genotype = bigint[]

export interface Solution {
    seeds: Genotype
    measures_raw: Float32Array
    measures_norm: Float32Array
    fitness: number
    competitionFitness?: number
    generation: number
}

function clamp(number: number, min: number, max: number) {
    return Math.max(min, Math.min(number, max))
}

export const archive_rows = 20
export const archive_cols = 42

const limits = [
    { min: 10, max: 200 },
    { min: 1.3, max: 3.0 },
]

export function measures_normalized(measures: Float32Array) {
    let m0_norm =
        (measures[0] - limits[0].min) / (limits[0].max - limits[0].min)
    let m1_norm =
        (measures[1] - limits[1].min) / (limits[1].max - limits[1].min)
    m0_norm = clamp(m0_norm, 0, 1)
    m1_norm = clamp(m1_norm, 0, 1)
    return new Float32Array([m0_norm, m1_norm])
}

export function archive_bin(solution: Solution) {
    let m0_norm = solution.measures_norm[0]
    let m1_norm = solution.measures_norm[1]
    let col = clamp(Math.floor(m0_norm * archive_cols), 0, archive_cols)
    let row = clamp(Math.floor(m1_norm * archive_rows), 0, archive_rows)
    return col * archive_rows + row
}

function calculate_dominated_novelty_scores(solutions: Solution[]) {
    // based on "Dominated Novelty Search", see: https://arxiv.org/abs/2502.00593v1
    const K = 5
    const N = solutions.length
    const M = solutions[0].measures_raw.length

    // sort by fitness (ascending)
    solutions.sort((a, b) => a.fitness - b.fitness)

    // pairwise squared distance
    let distances2 = new Float32Array(N * N)
    for (let i = 0; i < N; i++) {
        for (let j = i + 1; j < N; j++) {
            let dist2 = 0
            for (let m = 0; m < M; m++) {
                let diff =
                    solutions[i].measures_raw[m] - solutions[j].measures_raw[m]
                dist2 += diff * diff
            }
            distances2[j * N + i] = dist2
            distances2[i * N + j] = dist2
        }
    }

    for (let i = 0; i < N; i++) {
        let dists2 = distances2.slice(i * N + (i + 1), i * N + N)
        dists2.sort()
        if (dists2.length === 0) {
            solutions[i].competitionFitness = Infinity
        } else {
            let dists = dists2.slice(0, K).map((dist2) => Math.sqrt(dist2))
            solutions[i].competitionFitness =
                dists.reduce((a, b) => a + b, 0) / dists.length
            if (!isFinite(solutions[i].competitionFitness ?? NaN)) {
                console.log(solutions[i])
                throw new Error('competitionFitness not finite')
            }
        }
    }
}

function reevaluate_fitness(solutions: Solution[]) {
    for (let solution of solutions) {
        // random, but preserve older solutions (with fewer mutation steps)
        solution.fitness = -solution.generation + Math.random()
    }
}

export function novelty_search_reduce(
    solutions: Solution[],
    N: number,
): Solution[] {
    console.log('evaluating distances for', solutions.length)
    // let solutions = solutions.slice()
    reevaluate_fitness(solutions)
    calculate_dominated_novelty_scores(solutions)
    solutions.sort((a, b) => a.competitionFitness! - b.competitionFitness!)
    return solutions.slice(-N)
}
