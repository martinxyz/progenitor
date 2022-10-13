use super::{nn, wall_sim::Builders};

#[allow(clippy::excessive_precision)]
pub const HP: nn::Hyperparams = nn::Hyperparams {
    init_fac: 1.0,
    bias_fac: 0.1,
};

// optimized by CMA-ES, average depth: ~17.4 after 1000 steps
#[allow(clippy::excessive_precision)]
pub const PARAMS: [f32; Builders::PARAM_COUNT] = [
    1.755276216106271647e+01,
    -2.655213320962883206e+00,
    -3.766973205085216136e+01,
    -1.506272636022009515e+01,
    -2.191544042209774190e+01,
    -1.479125248674793092e+01,
    -1.299455324433507464e+01,
    -3.483598261532335272e+01,
    -3.347237221910778260e+01,
    -1.260090068189380474e+01,
    -3.149459628227665320e+01,
    9.331977589132264939e+00,
    -2.206756154360429711e+00,
    1.614760380040366528e+01,
    -1.780200061076597562e+00,
    -2.043504363903659637e+01,
    -9.527761094768345984e+00,
    1.276108169481820021e+01,
    -1.427426205930121661e+01,
    -3.104961515411757134e+01,
    2.622760510936669576e+01,
    -1.528171121939693933e+01,
    -2.020162404786054466e+01,
    -1.487040231135814183e+00,
    1.188903700627521154e+00,
    4.063700994021503732e+01,
    2.558815615941005372e+00,
    1.146696931550623155e+01,
    3.890878095819833682e+00,
    -2.734724069140317848e+00,
    -4.869702420795247733e+00,
    -2.446794516707143785e+01,
    2.297857234184082387e+01,
    -3.902160755972556405e+00,
    1.567748413178703115e-01,
    2.915190910880649078e+00,
    -3.711553525774665130e-01,
    3.824967507267098554e+00,
    6.486539814347199950e+00,
    2.470413208605599564e+01,
    -4.788337617434676119e+00,
    -9.415023695973172568e-01,
    1.633406118384994699e+01,
    -3.459138060294949302e+00,
    -1.078295407805632067e+01,
    -1.154729748950581758e+01,
    -1.403807394182747004e+01,
    1.931984041166756594e+00,
    -7.418723041936774898e+00,
    1.045609826136820963e+01,
    -2.304080005481219828e+01,
    -1.033251454878743125e+01,
    -1.358687406330225045e+01,
    -1.263424256881930141e+01,
    -3.476721756830483123e+00,
    -6.349369195898436402e+00,
    7.291800493288698171e+00,
    9.249675089122916249e-01,
    -1.746435485562247791e+01,
    3.108247799882985518e+01,
    4.335279336404190609e+00,
    -6.955528212736527216e+00,
    -6.728649099457980398e+00,
    -9.674827940881895572e+00,
    1.421948620248606687e+00,
    -9.515884948455882508e+00,
    -8.966882660508972425e-01,
    5.941518197131357759e+00,
    7.699766567707653220e-01,
    -7.991070883027141747e+00,
    -6.276549088648688546e+00,
    -6.797620031664986406e+00,
    -3.484333515896172351e+00,
    5.822253314711200112e+00,
    -3.574641371527477851e+00,
    -2.741299630049221392e+01,
    1.496759732945152344e+01,
    7.579131194348901479e-01,
    -1.576813952584122491e+00,
    3.507182634905766649e-01,
    -2.236085170203580219e+01,
    -1.408638656859329208e+00,
    -1.037119928851815054e+01,
    -6.695390598119003833e+00,
    -9.983162382204828766e-01,
    -1.119353485993281083e+01,
    1.554465430136809623e+01,
    -2.884705974813554175e+00,
    1.310645304368694530e+00,
    -9.400216837245263690e+00,
    1.309016326659835805e+01,
    1.444978182950726797e+01,
    -5.879822887636551698e+00,
    1.373259534337001497e+01,
    1.187746784911887765e+01,
    1.197589517983216467e+01,
    8.377627832830437882e+00,
    4.824797054929534368e-01,
    2.887425702007763206e+01,
    -1.501288840273449310e+01,
    1.159222960928316937e+01,
    -4.609085191371852019e+00,
    9.884328555247535064e+00,
    -1.256666573075734838e+01,
    1.856159344716970949e+01,
    2.159450813527844915e+01,
    -2.882980823668035875e+00,
    2.852601922232621945e+01,
    -8.088954605603341008e+00,
    -1.774197734934019621e+01,
    -1.335831926302518013e+01,
    -6.439523276366506011e+00,
    3.879412276524767123e+01,
    -2.259261930853659450e+01,
];
