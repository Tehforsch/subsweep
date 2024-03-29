use ordered_float::OrderedFloat;
use subsweep::units::Dimensionless;
use subsweep::units::Mass;
use subsweep::units::SourceRate;
use subsweep::units::Time;

fn linear_interpolate(x1: f64, x2: f64, y1: f64, y2: f64, x: f64) -> f64 {
    y1 + (x - x1) / (x2 - x1) * (y2 - y1)
}

pub fn bpass_lookup(age: Time, metallicity: Dimensionless, mass: Mass) -> SourceRate {
    let get_index = |bins: &[f64], value: f64| {
        bins.binary_search_by_key(&OrderedFloat(value), |x| OrderedFloat(*x))
            .map(|x| x + 1)
            .unwrap_or_else(|e| e)
    };
    let metallicity = metallicity.value();
    let age = age.in_years();

    let safety = 1.00001;
    let age = age.clamp(
        BPASS_AGE_BINS[0] * safety,
        BPASS_AGE_BINS[NUM_BPASS_AGES - 1] / safety,
    );
    let metallicity = metallicity.clamp(
        BPASS_METALLICITY_BINS[0] * safety,
        BPASS_METALLICITY_BINS[NUM_BPASS_METALLICITIES - 1] / safety,
    );

    let metallicity_index = get_index(&BPASS_METALLICITY_BINS, metallicity);
    let age_index = get_index(&BPASS_AGE_BINS, age);

    let l1 = linear_interpolate(
        BPASS_AGE_BINS[age_index - 1],
        BPASS_AGE_BINS[age_index],
        BPASS_TABLE[age_index - 1][metallicity_index - 1],
        BPASS_TABLE[age_index][metallicity_index - 1],
        age,
    );
    let l2 = linear_interpolate(
        BPASS_AGE_BINS[age_index - 1],
        BPASS_AGE_BINS[age_index],
        BPASS_TABLE[age_index - 1][metallicity_index],
        BPASS_TABLE[age_index][metallicity_index],
        age,
    );

    let source_rate_per_mass = linear_interpolate(
        BPASS_METALLICITY_BINS[metallicity_index - 1],
        BPASS_METALLICITY_BINS[metallicity_index],
        l1,
        l2,
        metallicity,
    );
    SourceRate::photons_per_second(source_rate_per_mass * mass.in_solar())
}

const NUM_BPASS_AGES: usize = 51;
const NUM_BPASS_METALLICITIES: usize = 13;

const BPASS_AGE_BINS: [f64; NUM_BPASS_AGES] = [
    1.00000000e+06,
    1.25892541e+06,
    1.58489319e+06,
    1.99526231e+06,
    2.51188643e+06,
    3.16227766e+06,
    3.98107171e+06,
    5.01187234e+06,
    6.30957344e+06,
    7.94328235e+06,
    1.00000000e+07,
    1.25892541e+07,
    1.58489319e+07,
    1.99526231e+07,
    2.51188643e+07,
    3.16227766e+07,
    3.98107171e+07,
    5.01187234e+07,
    6.30957344e+07,
    7.94328235e+07,
    1.00000000e+08,
    1.25892541e+08,
    1.58489319e+08,
    1.99526231e+08,
    2.51188643e+08,
    3.16227766e+08,
    3.98107171e+08,
    5.01187234e+08,
    6.30957344e+08,
    7.94328235e+08,
    1.00000000e+09,
    1.25892541e+09,
    1.58489319e+09,
    1.99526231e+09,
    2.51188643e+09,
    3.16227766e+09,
    3.98107171e+09,
    5.01187234e+09,
    6.30957344e+09,
    7.94328235e+09,
    1.00000000e+10,
    1.25892541e+10,
    1.58489319e+10,
    1.99526231e+10,
    2.51188643e+10,
    3.16227766e+10,
    3.98107171e+10,
    5.01187234e+10,
    6.30957344e+10,
    7.94328235e+10,
    1.00000000e+11,
];

const BPASS_METALLICITY_BINS: [f64; NUM_BPASS_METALLICITIES] = [
    1.0e-05, 1.0e-04, 1.0e-03, 2.0e-03, 3.0e-03, 4.0e-03, 6.0e-03, 8.0e-03, 1.0e-02, 1.4e-02,
    2.0e-02, 3.0e-02, 4.0e-02,
];

const BPASS_TABLE: [[f64; NUM_BPASS_METALLICITIES]; NUM_BPASS_AGES] = [
    [
        4.22119750e+46,
        4.11037266e+46,
        4.00498367e+46,
        4.04825020e+46,
        4.03299634e+46,
        4.01878889e+46,
        3.99539130e+46,
        3.91022835e+46,
        3.85809308e+46,
        3.66317441e+46,
        3.56523816e+46,
        3.41850883e+46,
        3.34144693e+46,
    ],
    [
        4.71532649e+46,
        4.61303242e+46,
        4.69053909e+46,
        4.65785492e+46,
        4.55349796e+46,
        4.47576653e+46,
        4.33997913e+46,
        4.18178708e+46,
        4.04890650e+46,
        3.63717334e+46,
        3.42128803e+46,
        3.02583642e+46,
        2.86809438e+46,
    ],
    [
        5.03932484e+46,
        5.02587803e+46,
        5.02914570e+46,
        4.90438712e+46,
        4.73979763e+46,
        4.63427630e+46,
        4.44168652e+46,
        4.26563559e+46,
        4.03772954e+46,
        3.53628832e+46,
        3.24544345e+46,
        2.71690607e+46,
        2.41658159e+46,
    ],
    [
        5.68408989e+46,
        5.62260343e+46,
        5.29877273e+46,
        5.09791117e+46,
        4.87241936e+46,
        4.68992976e+46,
        4.41623005e+46,
        4.14455405e+46,
        3.76764225e+46,
        3.28340957e+46,
        2.85948739e+46,
        2.14944739e+46,
        1.80216454e+46,
    ],
    [
        6.41617828e+46,
        6.19966678e+46,
        5.37002134e+46,
        5.03664547e+46,
        4.68593767e+46,
        4.37378019e+46,
        3.94872536e+46,
        3.53350359e+46,
        3.05127468e+46,
        2.54284498e+46,
        2.16874301e+46,
        1.61456943e+46,
        1.40691116e+46,
    ],
    [
        6.15016125e+46,
        5.68894406e+46,
        4.93842423e+46,
        4.53376813e+46,
        4.16668159e+46,
        3.83133624e+46,
        3.33895114e+46,
        2.97296731e+46,
        2.58550755e+46,
        2.02472067e+46,
        1.89161127e+46,
        1.34468187e+46,
        1.18730797e+46,
    ],
    [
        4.61709710e+46,
        4.21860019e+46,
        3.39392286e+46,
        3.20504403e+46,
        2.89853264e+46,
        2.64461654e+46,
        2.28410554e+46,
        2.07392616e+46,
        1.79417724e+46,
        1.40071755e+46,
        1.23997361e+46,
        8.00472755e+45,
        6.52153030e+45,
    ],
    [
        3.51946810e+46,
        3.07958288e+46,
        2.17694085e+46,
        1.88350530e+46,
        1.77991761e+46,
        1.67236691e+46,
        1.28077827e+46,
        1.11477113e+46,
        9.49416240e+45,
        7.54103983e+45,
        6.69250874e+45,
        4.44995022e+45,
        3.43280950e+45,
    ],
    [
        2.43980538e+46,
        2.03603461e+46,
        1.26748932e+46,
        1.04268938e+46,
        9.19249728e+45,
        8.80358864e+45,
        6.54030793e+45,
        5.87317024e+45,
        4.91396938e+45,
        3.88025776e+45,
        3.20004386e+45,
        2.28463891e+45,
        1.90397743e+45,
    ],
    [
        1.49216673e+46,
        1.20675998e+46,
        7.41644827e+45,
        5.68398165e+45,
        4.72498999e+45,
        4.22958317e+45,
        2.94744274e+45,
        2.66931611e+45,
        2.40242705e+45,
        2.06304292e+45,
        1.95545602e+45,
        1.37056891e+45,
        1.12821580e+45,
    ],
    [
        7.97926599e+45,
        6.62285390e+45,
        4.06765326e+45,
        3.16070647e+45,
        2.64032209e+45,
        2.22389625e+45,
        1.56121376e+45,
        1.43346521e+45,
        1.34038299e+45,
        1.13779828e+45,
        1.02595726e+45,
        8.42882568e+44,
        7.15384640e+44,
    ],
    [
        3.44747799e+45,
        3.01016838e+45,
        2.15194071e+45,
        1.74071204e+45,
        1.47289599e+45,
        1.22616254e+45,
        8.55575552e+44,
        7.92073846e+44,
        7.50295387e+44,
        6.57238966e+44,
        5.96598615e+44,
        4.71049791e+44,
        4.44519850e+44,
    ],
    [
        1.43656431e+45,
        1.06382874e+45,
        7.27078135e+44,
        7.48126778e+44,
        7.46274043e+44,
        6.91280610e+44,
        5.10760313e+44,
        4.34220170e+44,
        4.09621470e+44,
        3.83611896e+44,
        3.24700430e+44,
        2.93562325e+44,
        2.68246493e+44,
    ],
    [
        5.35868972e+44,
        4.01984200e+44,
        4.00903516e+44,
        3.98680699e+44,
        3.72247304e+44,
        3.71294029e+44,
        2.60254530e+44,
        2.55303518e+44,
        2.42494240e+44,
        2.13922010e+44,
        2.05130766e+44,
        2.13739519e+44,
        1.76862616e+44,
    ],
    [
        1.36932260e+44,
        1.10361636e+44,
        1.80116319e+44,
        1.77780725e+44,
        1.68691328e+44,
        1.60071347e+44,
        1.84877419e+44,
        1.56624059e+44,
        1.54746400e+44,
        1.46823261e+44,
        1.21395508e+44,
        1.15719324e+44,
        1.36296583e+44,
    ],
    [
        6.67152621e+43,
        7.30339884e+43,
        1.38155905e+44,
        1.33508951e+44,
        1.21494566e+44,
        1.13977391e+44,
        1.12566663e+44,
        1.10005446e+44,
        1.01182794e+44,
        9.13568389e+43,
        9.60415804e+43,
        9.03594703e+43,
        8.60693400e+43,
    ],
    [
        3.10744807e+43,
        5.19661360e+43,
        9.25227573e+43,
        9.27007195e+43,
        8.45597432e+43,
        7.91328774e+43,
        7.46298467e+43,
        7.46872339e+43,
        7.43239620e+43,
        7.72094748e+43,
        6.72912324e+43,
        6.37512210e+43,
        5.58040527e+43,
    ],
    [
        2.00429333e+43,
        3.92430557e+43,
        7.16720120e+43,
        7.41329030e+43,
        6.09547907e+43,
        6.26347610e+43,
        6.36338617e+43,
        5.81174191e+43,
        5.66675910e+43,
        4.55998269e+43,
        4.92656943e+43,
        4.37972655e+43,
        4.40449696e+43,
    ],
    [
        1.21286082e+43,
        2.56007095e+43,
        5.15926471e+43,
        4.67239833e+43,
        5.36252992e+43,
        4.13193033e+43,
        3.93536180e+43,
        3.98222778e+43,
        3.35963845e+43,
        4.17278149e+43,
        4.51298955e+43,
        3.25689392e+43,
        3.45316826e+43,
    ],
    [
        7.47763970e+42,
        1.78388785e+43,
        3.31821548e+43,
        3.44588055e+43,
        3.30695898e+43,
        2.58686516e+43,
        2.65124181e+43,
        3.00544385e+43,
        2.16713642e+43,
        2.51872775e+43,
        2.76536724e+43,
        2.64588101e+43,
        2.41363779e+43,
    ],
    [
        4.62724383e+42,
        1.24037120e+43,
        2.27643240e+43,
        2.35645850e+43,
        2.85375102e+43,
        2.65318664e+43,
        2.39084134e+43,
        1.82840119e+43,
        1.72266723e+43,
        1.99992558e+43,
        1.82586719e+43,
        1.64877034e+43,
        1.51682600e+43,
    ],
    [
        3.19897702e+42,
        8.29132268e+42,
        1.49243526e+43,
        1.47381267e+43,
        1.98596440e+43,
        1.56797933e+43,
        1.27773892e+43,
        1.81809748e+43,
        1.28408206e+43,
        1.38635845e+43,
        1.19748191e+43,
        1.13057919e+43,
        1.14319363e+43,
    ],
    [
        1.42571004e+42,
        4.20043221e+42,
        6.94480142e+42,
        7.89145110e+42,
        9.38480262e+42,
        9.20568503e+42,
        9.62880153e+42,
        9.97219965e+42,
        7.63609719e+42,
        8.33741533e+42,
        9.46318661e+42,
        6.83023453e+42,
        7.51494860e+42,
    ],
    [
        1.16516529e+42,
        1.84580331e+42,
        4.31199597e+42,
        4.88973172e+42,
        6.33320593e+42,
        7.02891564e+42,
        6.20859297e+42,
        6.10980136e+42,
        4.45253158e+42,
        4.29615937e+42,
        4.49995937e+42,
        2.96952821e+42,
        3.42921902e+42,
    ],
    [
        9.41626206e+41,
        1.09975181e+42,
        2.07738399e+42,
        3.58708980e+42,
        3.47788716e+42,
        4.54769168e+42,
        4.22271497e+42,
        4.02883431e+42,
        2.70802139e+42,
        3.15433777e+42,
        4.19489196e+42,
        2.21357118e+42,
        2.60037719e+42,
    ],
    [
        4.57040673e+41,
        4.85833225e+41,
        1.42970962e+42,
        1.87696509e+42,
        2.40324158e+42,
        2.59724294e+42,
        2.80448625e+42,
        2.60383243e+42,
        1.54649774e+42,
        1.72422815e+42,
        1.80750500e+42,
        1.31671863e+42,
        1.29669986e+42,
    ],
    [
        2.67153281e+41,
        4.45239254e+41,
        8.20671059e+41,
        1.10776758e+42,
        1.64765004e+42,
        2.09655149e+42,
        1.87523511e+42,
        1.52349528e+42,
        8.73627114e+41,
        5.59539778e+41,
        5.84606091e+41,
        4.41348797e+41,
        5.33034490e+41,
    ],
    [
        1.54298547e+41,
        1.87802909e+41,
        4.46365771e+41,
        4.98133224e+41,
        4.55269293e+41,
        9.86769742e+41,
        7.56289828e+41,
        6.84900983e+41,
        4.18965808e+41,
        3.98065667e+41,
        4.80990294e+41,
        3.97201422e+41,
        4.40138316e+41,
    ],
    [
        9.05557685e+40,
        1.22586760e+41,
        2.10709695e+41,
        2.51112576e+41,
        2.80410227e+41,
        5.73480658e+41,
        5.89214015e+41,
        4.61620366e+41,
        2.59988749e+41,
        2.05468002e+41,
        2.67563553e+41,
        1.97659096e+41,
        1.99002155e+41,
    ],
    [
        6.36116589e+40,
        7.65198671e+40,
        1.51110467e+41,
        1.82618860e+41,
        1.97049051e+41,
        2.86279253e+41,
        2.55367860e+41,
        2.34709739e+41,
        1.67140036e+41,
        2.06437441e+41,
        1.91975120e+41,
        1.40978701e+41,
        1.67833640e+41,
    ],
    [
        4.51931964e+40,
        1.04801192e+41,
        1.10464926e+41,
        8.87750301e+40,
        1.07765927e+41,
        1.36815853e+41,
        1.44357455e+41,
        9.57168856e+40,
        1.19922361e+41,
        1.06610473e+41,
        1.40634892e+41,
        1.58812318e+41,
        1.76099183e+41,
    ],
    [
        5.44499671e+40,
        5.10587634e+40,
        1.26850437e+41,
        9.24934930e+40,
        7.92907207e+40,
        8.80519071e+40,
        9.86810820e+40,
        6.46766139e+40,
        3.45349361e+40,
        9.26713024e+40,
        7.85000012e+40,
        1.08572729e+41,
        1.06065623e+41,
    ],
    [
        4.19739199e+40,
        3.80925425e+40,
        5.80847063e+40,
        8.74346949e+40,
        8.08075641e+40,
        8.40021806e+40,
        9.86406473e+40,
        4.29066901e+40,
        5.70136326e+40,
        4.30624165e+40,
        4.56865090e+40,
        3.10298146e+40,
        6.06444983e+40,
    ],
    [
        2.56927508e+40,
        2.65511466e+40,
        4.81643625e+40,
        5.14329677e+40,
        4.93314939e+40,
        5.93958355e+40,
        5.90770070e+40,
        5.81412012e+40,
        5.93074316e+40,
        5.49378929e+40,
        6.87266023e+40,
        6.94893786e+40,
        5.61387070e+40,
    ],
    [
        2.19101711e+40,
        2.29629043e+40,
        3.55257699e+40,
        3.50805060e+40,
        3.62979491e+40,
        4.04097690e+40,
        4.39791617e+40,
        3.99443305e+40,
        4.68166255e+40,
        6.45213251e+40,
        5.41870132e+40,
        6.42915116e+40,
        6.98036706e+40,
    ],
    [
        1.91215247e+40,
        1.71072374e+40,
        2.12961452e+40,
        2.79556747e+40,
        2.72416205e+40,
        2.96713563e+40,
        3.23184865e+40,
        3.11271202e+40,
        3.15995516e+40,
        3.94631997e+40,
        4.44039784e+40,
        5.46583985e+40,
        4.45581567e+40,
    ],
    [
        2.20111632e+40,
        1.65783623e+40,
        1.94917221e+40,
        1.61622441e+40,
        1.96027851e+40,
        2.08862086e+40,
        2.18689143e+40,
        2.86895973e+40,
        2.89985972e+40,
        3.04897614e+40,
        3.37148298e+40,
        3.91880262e+40,
        5.10595402e+40,
    ],
    [
        7.62626795e+39,
        6.62110570e+39,
        1.00405679e+40,
        1.61077599e+40,
        1.71784821e+40,
        1.79484935e+40,
        1.65900227e+40,
        1.98702136e+40,
        2.56108609e+40,
        2.19137694e+40,
        3.08038639e+40,
        3.31004624e+40,
        3.49052862e+40,
    ],
    [
        1.90473697e+40,
        1.01185795e+40,
        9.59908230e+39,
        1.09763332e+40,
        1.02849084e+40,
        9.73617795e+39,
        1.61013264e+40,
        1.71529740e+40,
        1.39482550e+40,
        1.86762651e+40,
        2.07796692e+40,
        2.22639993e+40,
        2.52821647e+40,
    ],
    [
        1.51369396e+40,
        1.17473139e+40,
        1.01316930e+40,
        6.94983752e+39,
        8.12420867e+39,
        1.17004954e+40,
        1.04930616e+40,
        1.03284011e+40,
        1.39822944e+40,
        1.63416255e+40,
        1.36153273e+40,
        2.00647138e+40,
        8.51297781e+38,
    ],
    [
        2.31950011e+39,
        4.79684878e+39,
        5.75308051e+39,
        9.50867685e+39,
        9.40270815e+39,
        9.54624983e+39,
        7.91204691e+39,
        9.41195956e+39,
        1.10647604e+40,
        9.43570018e+39,
        1.44631455e+40,
        1.32768176e+40,
        4.19063321e+41,
    ],
    [
        2.39937511e+40,
        1.25408099e+40,
        7.90569240e+39,
        5.82076245e+39,
        6.79126890e+39,
        4.82566656e+39,
        9.37457672e+39,
        9.31702868e+39,
        8.26777634e+39,
        9.71026960e+39,
        9.40762884e+39,
        1.14475349e+39,
        3.69984650e+41,
    ],
    [
        3.17441593e+39,
        2.85735622e+39,
        3.17689315e+39,
        5.67304913e+39,
        7.49265412e+39,
        8.04705532e+39,
        7.04618841e+39,
        7.35329145e+39,
        8.76497584e+39,
        8.37279266e+39,
        8.16236409e+39,
        2.19052203e+38,
        2.12661176e+41,
    ],
    [
        4.09890563e+38,
        1.25190489e+39,
        3.96847776e+39,
        5.08075277e+39,
        4.00879424e+39,
        4.20904882e+39,
        5.19996384e+39,
        5.88170752e+39,
        6.43041869e+39,
        7.25838563e+39,
        3.20376518e+39,
        1.57708542e+41,
        2.31312836e+41,
    ],
    [
        9.65885757e+38,
        1.54019787e+39,
        2.69853705e+39,
        3.06286069e+39,
        3.21678315e+39,
        5.14606404e+39,
        6.07760493e+39,
        6.18172595e+39,
        6.69549700e+39,
        6.20776245e+39,
        1.35861221e+37,
        2.86711372e+41,
        1.54125926e+41,
    ],
    [
        2.35770217e+39,
        5.01508365e+39,
        2.66883106e+39,
        2.78604690e+39,
        3.00885911e+39,
        3.31240904e+39,
        3.60545853e+39,
        3.72070525e+39,
        3.75042319e+39,
        2.22344448e+39,
        8.61628347e+40,
        3.43510263e+41,
        7.39207065e+40,
    ],
    [
        1.55783687e+40,
        9.68313853e+39,
        1.40060963e+40,
        4.55702621e+39,
        2.70825874e+39,
        2.99977114e+39,
        3.31856730e+39,
        3.47890698e+39,
        1.69092083e+39,
        5.83819678e+35,
        2.55250457e+41,
        1.39616117e+41,
        6.25660370e+40,
    ],
    [
        2.70326969e+41,
        1.45126708e+41,
        6.30203284e+40,
        4.50004224e+40,
        2.67887440e+39,
        2.40169560e+39,
        2.48193290e+39,
        2.99556701e+36,
        1.36918683e+35,
        9.48034859e+40,
        2.52142186e+41,
        1.47806413e+41,
        1.60088651e+41,
    ],
    [
        4.19219774e+41,
        3.61438684e+41,
        3.03860333e+41,
        1.28591345e+41,
        6.17727532e+40,
        3.11986919e+37,
        3.21818917e+38,
        2.49209825e+40,
        8.13440151e+40,
        2.65639704e+41,
        1.86641011e+41,
        6.15264101e+40,
        4.50143395e+40,
    ],
    [
        9.96352739e+39,
        1.55752189e+41,
        3.32392272e+41,
        2.52454162e+41,
        2.68280569e+41,
        9.85563485e+40,
        7.29400656e+40,
        3.37828143e+41,
        1.42512075e+41,
        2.21478991e+41,
        9.50992428e+40,
        1.09893312e+41,
        1.07608542e+40,
    ],
    [
        1.74275203e+37,
        1.19014479e+41,
        1.82704975e+30,
        1.04957724e+41,
        4.90569163e+37,
        1.50014482e+41,
        1.38429279e+41,
        1.37787275e+41,
        1.05340211e+41,
        9.25344709e+40,
        7.27523308e+40,
        8.65606930e+40,
        7.26148342e+39,
    ],
];

#[cfg(test)]
mod tests {
    use subsweep::prelude::Float;
    use subsweep::units::Dimensionless;
    use subsweep::units::Mass;
    use subsweep::units::Time;

    use super::bpass_lookup;

    pub fn assert_float_is_close(x: Float, y: Float) {
        assert!(((x - y) / (x.abs() + y.abs())).abs() < 1e-5, "{} {}", x, y)
    }

    #[test]
    fn bpass() {
        let check_is_close = |age_in_years, metallicity, desired_value_photons_per_s| {
            let source = bpass_lookup(
                Time::years(age_in_years),
                Dimensionless::dimensionless(metallicity),
                Mass::solar(1.0),
            );
            assert_float_is_close(source.in_photons_per_second(), desired_value_photons_per_s);
        };
        check_is_close(1e6, 1e-5, 4.22119750e+46);
        check_is_close(1e6, 3.5e-2, 3.37997788e+46);
        check_is_close(6.30957344e+10, 1e-5, 4.19219774e+41);
    }
}
